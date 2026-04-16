#![allow(dead_code)]
// Callers (transcription::run_on_session) wire up in Phase 5.

//! Thin wrapper around `whisper-rs` for English-only segment transcription.
//!
//! The public contract here is intentionally narrow:
//! - input is mono 16 kHz f32 PCM
//! - output is typed, timestamped segments with a fixed speaker label
//! - model loading chooses the platform-appropriate accelerator when compiled

use anyhow::{Context, Result};
use std::env;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub start_sec: f64,
    pub end_sec: f64,
    pub speaker: Speaker,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    You,
    Meeting,
}

impl Speaker {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::You => "You",
            Self::Meeting => "Meeting",
        }
    }
}

pub struct Transcriber {
    ctx: WhisperContext,
    pub accelerator: &'static str,
}

impl Transcriber {
    pub fn load(model_path: &Path) -> Result<Self> {
        let prefer_gpu = should_prefer_gpu();

        if prefer_gpu {
            match load_context(model_path, true) {
                Ok(ctx) => {
                    let accelerator = compiled_accelerator();
                    tracing::info!(
                        accelerator,
                        model_path = %model_path.display(),
                        "loaded whisper context"
                    );
                    return Ok(Self { ctx, accelerator });
                }
                Err(err) => {
                    // Some backends return a recoverable init error. Others
                    // can crash outright; tests default to CPU to avoid
                    // depending on unstable native GPU startup.
                    tracing::warn!(
                        error = %err,
                        model_path = %model_path.display(),
                        "GPU whisper init failed, retrying on CPU"
                    );
                }
            }
        }

        let ctx = load_context(model_path, false)?;
        tracing::info!(
            accelerator = "cpu",
            model_path = %model_path.display(),
            "loaded whisper context"
        );
        Ok(Self {
            ctx,
            accelerator: "cpu",
        })
    }

    /// Transcribe a single mono 16 kHz f32 buffer. Returns segments sorted by
    /// start time, each labeled with the provided speaker.
    pub fn transcribe(&self, samples_16k_mono: &[f32], speaker: Speaker) -> Result<Vec<Segment>> {
        if samples_16k_mono.is_empty() {
            return Ok(Vec::new());
        }

        let mut state = self
            .ctx
            .create_state()
            .context("failed to create whisper state")?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_no_speech_thold(0.6);
        params.set_n_threads(available_threads());

        state
            .full(params, samples_16k_mono)
            .context("whisper inference failed")?;

        let mut out = Vec::new();
        for raw in state.as_iter() {
            let text = raw
                .to_str_lossy()
                .context("failed to read whisper segment text")?
                .trim()
                .to_string();
            if text.is_empty() {
                continue;
            }

            // whisper timestamps are in 10 ms units.
            let start_sec = raw.start_timestamp() as f64 / 100.0;
            let end_sec = raw.end_timestamp() as f64 / 100.0;

            out.push(Segment {
                start_sec,
                end_sec,
                speaker,
                text,
            });
        }

        Ok(out)
    }
}

fn load_context(model_path: &Path, use_gpu: bool) -> Result<WhisperContext> {
    let mut cparams = WhisperContextParameters::default();
    cparams.use_gpu(use_gpu);

    WhisperContext::new_with_params(model_path, cparams)
        .with_context(|| format!("failed to load whisper model at {}", model_path.display()))
}

fn available_threads() -> i32 {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(i32::MAX as usize) as i32
}

fn should_prefer_gpu() -> bool {
    match env::var("DOMINO_WHISPER_USE_GPU") {
        Ok(value) => parse_bool_env(&value).unwrap_or_else(|| {
            tracing::warn!(
                value,
                "unrecognized DOMINO_WHISPER_USE_GPU value, using default"
            );
            default_prefer_gpu()
        }),
        Err(_) => default_prefer_gpu(),
    }
}

fn default_prefer_gpu() -> bool {
    // `cargo test` on this repo currently runs through a Swift runtime shim on
    // macOS that makes the native GPU path unreliable. Keep the smoke tests on
    // CPU by default, but let normal builds prefer the compiled accelerator.
    !cfg!(test) && !matches!(compiled_accelerator(), "cpu")
}

fn parse_bool_env(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn compiled_accelerator() -> &'static str {
    "metal"
}

#[cfg(target_os = "windows")]
fn compiled_accelerator() -> &'static str {
    "vulkan"
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn compiled_accelerator() -> &'static str {
    "cpu"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcription::model::{models_dir, MODEL_FILENAME};

    #[test]
    fn test_speaker_as_str() {
        assert_eq!(Speaker::You.as_str(), "You");
        assert_eq!(Speaker::Meeting.as_str(), "Meeting");
    }

    #[test]
    fn test_available_threads_positive() {
        assert!(available_threads() > 0);
    }

    #[test]
    fn test_compiled_accelerator_non_empty() {
        assert!(!compiled_accelerator().is_empty());
    }

    #[test]
    fn test_parse_bool_env() {
        assert_eq!(parse_bool_env("1"), Some(true));
        assert_eq!(parse_bool_env("true"), Some(true));
        assert_eq!(parse_bool_env("ON"), Some(true));
        assert_eq!(parse_bool_env("0"), Some(false));
        assert_eq!(parse_bool_env("false"), Some(false));
        assert_eq!(parse_bool_env("off"), Some(false));
        assert_eq!(parse_bool_env("maybe"), None);
    }

    #[test]
    #[ignore = "requires ~/.domino/models/ggml-small.en.bin"]
    fn test_model_loads() {
        let model_path = models_dir().unwrap().join(MODEL_FILENAME);
        assert!(
            model_path.exists(),
            "missing model fixture at {}",
            model_path.display()
        );

        let transcriber = Transcriber::load(&model_path).unwrap();
        assert!(
            transcriber.accelerator == "cpu" || transcriber.accelerator == compiled_accelerator()
        );
    }
}
