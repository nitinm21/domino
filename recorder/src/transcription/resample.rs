#![allow(dead_code)]
// Callers (transcription::run_on_session) wire up in Phase 5.

//! Mono f32 resampling, sized for the whisper input contract
//! (48 kHz capture → 16 kHz inference).

use anyhow::{Context, Result};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

const CHUNK_SIZE: usize = 1024;

/// Resample a mono f32 buffer from `from_hz` to `to_hz`.
///
/// Returns a buffer of length ≈ `input.len() * to_hz / from_hz`. The trailing
/// partial chunk is zero-padded, which introduces a handful of near-zero
/// output samples at the end — inconsequential for transcription.
pub fn resample_mono(input: &[f32], from_hz: u32, to_hz: u32) -> Result<Vec<f32>> {
    if from_hz == to_hz {
        return Ok(input.to_vec());
    }
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let ratio = to_hz as f64 / from_hz as f64;
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 160,
        window: WindowFunction::BlackmanHarris2,
    };
    let mut resampler = SincFixedIn::<f32>::new(ratio, 1.0, params, CHUNK_SIZE, 1)
        .context("failed to build resampler")?;

    let expected_out = (input.len() as f64 * ratio).ceil() as usize;
    let mut out: Vec<f32> = Vec::with_capacity(expected_out + CHUNK_SIZE);

    let mut pos = 0;
    while pos + CHUNK_SIZE <= input.len() {
        let chunk: &[f32] = &input[pos..pos + CHUNK_SIZE];
        let frames = resampler
            .process(&[chunk], None)
            .context("resampler process failed")?;
        out.extend_from_slice(&frames[0]);
        pos += CHUNK_SIZE;
    }

    // Tail: zero-pad the final partial chunk to CHUNK_SIZE so the resampler
    // keeps its fixed-size contract.
    if pos < input.len() {
        let mut tail = vec![0.0f32; CHUNK_SIZE];
        tail[..input.len() - pos].copy_from_slice(&input[pos..]);
        let frames = resampler
            .process(&[tail.as_slice()], None)
            .context("resampler process (tail) failed")?;
        out.extend_from_slice(&frames[0]);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(buf: &[f32]) -> f32 {
        if buf.is_empty() {
            return 0.0;
        }
        let sum: f32 = buf.iter().map(|x| x * x).sum();
        (sum / buf.len() as f32).sqrt()
    }

    #[test]
    fn test_identity_when_rates_match() {
        let input: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let out = resample_mono(&input, 48_000, 48_000).unwrap();
        assert_eq!(out, input);
    }

    #[test]
    fn test_empty_input() {
        let out = resample_mono(&[], 48_000, 16_000).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn test_silence_48k_to_16k_length() {
        let input = vec![0.0f32; 48_000];
        let out = resample_mono(&input, 48_000, 16_000).unwrap();
        let diff = (out.len() as i64 - 16_000).abs();
        // SincFixedIn + zero-padded tail can over/undershoot by a chunk-ratio
        // worth of samples; ±400 is well within what whisper tolerates.
        assert!(
            diff < 400,
            "silence length {} not near 16000 (diff {})",
            out.len(),
            diff
        );
        // Silence in → silence out.
        assert!(
            rms(&out) < 1e-4,
            "silence produced non-silence: rms={}",
            rms(&out)
        );
    }

    #[test]
    fn test_sine_rms_preserved_48k_to_16k() {
        let n = 48_000;
        let freq = 440.0f32;
        let input: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / 48_000.0).sin() * 0.5)
            .collect();
        let in_rms = rms(&input);

        let out = resample_mono(&input, 48_000, 16_000).unwrap();
        // Ignore the initial ~sinc_len/2 samples which can be attenuated by
        // the filter warming up from zero state.
        let warmup = 200;
        let body = &out[warmup..out.len().saturating_sub(warmup)];
        let out_rms = rms(body);

        // 440 Hz is well below the 16 kHz Nyquist (8 kHz), so RMS should be
        // preserved to within ~20%.
        assert!(
            (out_rms - in_rms).abs() / in_rms < 0.2,
            "RMS drift too large: in={in_rms}, out={out_rms}"
        );
    }

    #[test]
    fn test_output_length_close_to_ratio_for_arbitrary_size() {
        // Pick a size that's not a clean multiple of the chunk size to make
        // sure the tail handling doesn't blow up.
        let n = 48_000 + 123;
        let input = vec![0.0f32; n];
        let out = resample_mono(&input, 48_000, 16_000).unwrap();
        let expected = (n as f64 * 16_000.0 / 48_000.0) as i64;
        let diff = (out.len() as i64 - expected).abs();
        assert!(
            diff < 400,
            "output {} not near {} (diff {})",
            out.len(),
            expected,
            diff
        );
    }
}
