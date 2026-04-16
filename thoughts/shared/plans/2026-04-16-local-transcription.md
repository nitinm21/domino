# Local Transcription on `/record stop` — Implementation Plan

## Overview

Add automatic, offline, English-only transcription to the Domino recorder. After `/record stop` finalizes `meeting.opus`, the same CLI invocation decodes it, transcribes the mic (Left) and system (Right) channels independently using `whisper.cpp` via `whisper-rs` with the `ggml-small.en` model, merges the segments by start time, and writes `transcript.json` into the session directory. Transcription is blocking with progress streamed to stdout, GPU-accelerated (Metal on macOS, Vulkan on Windows) with CPU fallback, and never exposed to the user as a separate command.

## Current State Analysis

The recorder today is a capture-only daemon. The relevant pieces:

- **`recorder/src/main.rs:37–140`** — `cmd_start()` forks a detached daemon that runs mic + system capture and the Opus encoder until SIGTERM, then exits. The parent returns immediately with session JSON on stdout.
- **`recorder/src/main.rs:142–155`** — `cmd_stop()` calls `session::stop_session()`, then prints the saved file path and size. This is the hook we extend for transcription.
- **`recorder/src/session.rs:136–167`** — `stop_session()` sends SIGTERM to the daemon, waits up to 5s for it to exit (SIGKILL fallback), and removes the PID file. Returns `SessionInfo` containing `session_dir`.
- **`recorder/src/audio/encoder.rs:87–248`** — Opus encoder loop. On SIGTERM drain, writes a final packet with `PacketWriteEndInfo::EndStream` (line 210–223) so the Ogg file is valid when the daemon exits. This means by the time `stop_session()` returns, `meeting.opus` is finalized and ready to decode.
- **`recorder/src/session.rs:14–28`** — `~/.domino/` is the top-level data dir. Recordings live in `~/.domino/recordings/<session>/`. Models will live in `~/.domino/models/`.
- **`recorder/Cargo.toml:11–28`** — current deps: `cpal`, `ringbuf`, `audiopus`, `ogg`, plus `screencapturekit` on macOS.

What's missing: audio decode, resampling, whisper integration, model download/verification, transcript serialization, and the `cmd_stop()` wiring. All additive — no existing code needs to change its behavior during capture.

## Desired End State

After `/record stop` completes, the session directory contains:

```
~/.domino/recordings/2026-04-16-1423/
├── meeting.opus          (existing — stereo Opus, unchanged)
├── recorder.log          (existing — daemon's stderr)
├── transcript.json       (NEW — structured segments with speaker labels)
└── transcription.log     (NEW — raw tracing output from the transcription run)
```

`transcript.json` schema (version 1):

```json
{
  "version": 1,
  "audio_file": "meeting.opus",
  "duration_sec": 2832.4,
  "model": "ggml-small.en",
  "model_sha256": "1be3a9b2df72efb7…",
  "language": "en",
  "transcribed_at": "2026-04-16T14:24:12-07:00",
  "transcription_wall_sec": 118.3,
  "accelerator": "metal",
  "segments": [
    { "start": 0.00,  "end": 3.41, "speaker": "You",     "text": "Hey, thanks for joining." },
    { "start": 3.52,  "end": 7.11, "speaker": "Meeting", "text": "Yeah, happy to." }
  ]
}
```

Speaker values are exactly `"You"` or `"Meeting"` — no other values in v1. Segments are sorted by ascending `start`. Times are seconds from the start of the recording (audio t=0).

### Verification of the end state

- Record a 60-second meeting where the user speaks for the first 30s and a YouTube video plays for the last 30s.
- Run `/record stop`.
- Assert `transcript.json` exists, validates against the schema above, has segments labeled `"You"` in the first half and `"Meeting"` in the second half, and `duration_sec` is within ±0.5s of 60.
- Delete `~/.domino/models/ggml-small.en.bin`, record a new session, stop. Observe progress bar, SHA256 check, then successful transcription.

### Key Discoveries

- **Daemon finalizes the file before exiting**: `recorder/src/audio/encoder.rs:210–223` writes `EndStream` on the final packet after `shutdown && drained`. This means `meeting.opus` is always valid-Ogg by the time `stop_session()` returns — we can decode it immediately in the stop process without a coordination dance.
- **`stop_session()` runs in the user's terminal**: `recorder/src/main.rs:142–155` executes in the *CLI* process (not the daemon), so stdout is connected to the user's terminal. Streaming transcription progress to stdout is trivial — no IPC or log tailing needed.
- **Session dir path is already returned**: `SessionInfo.session_dir` at `recorder/src/session.rs:7–12` is the canonical place to put transcript artifacts.
- **`~/.domino/` is already created with mode 0o700**: `recorder/src/session.rs:30–41`. We can reuse `ensure_domino_dir()` when creating the models dir and will mirror its permission logic.
- **Whisper needs 16 kHz mono f32**: our Opus is 48 kHz stereo. We decode with `symphonia` (pure-Rust Opus/Ogg decoder, no ffmpeg), split stereo into two mono channels, then resample each from 48→16 kHz with `rubato` before feeding to whisper-rs.

## What We're NOT Doing

Listed explicitly to prevent scope creep:

- **No intra-meeting diarization.** The stereo split gives us "You" vs. "Meeting" for free. True speaker identification within the meeting channel (Bob vs. Alice) is feasible via `sherpa-onnx` + pyannote ONNX models, but adds a new runtime dependency (`onnxruntime`), ~50 MB of additional models, and measurable accuracy risk. Deferred to a future plan.
- **No word-level timestamps.** Segment-level only. Word timestamps roughly double transcription time and have no v1 consumer.
- **No language detection / multi-language.** `small.en` is English-only by construction. If users record non-English meetings in v1, they get English approximations; accepted as a known limitation.
- **No streaming / live transcription.** Transcription runs after `/record stop`, not during capture. Live captions are a different product surface.
- **No transcription subcommand.** `domino-recorder transcribe <session-dir>` is not exposed. An internal entry point (`transcription::run_on_session(path)`) exists for tests and for resume-after-Ctrl-C scenarios, but is not reachable from the public CLI.
- **No retry on transcription failure.** If whisper errors out, we log it, print a clear message, and exit non-zero. The audio file remains on disk. No automatic retry, no background retry queue.
- **No audio deletion after transcription.** `meeting.opus` stays on disk alongside the transcript. Cleanup policies are a separate future concern.
- **No transcript format variants** (txt/vtt/md). `transcript.json` only. Others are cheap to derive later from the JSON; no point baking them in before we know who consumes them.
- **No changes to the capture daemon.** Zero code under `recorder/src/audio/` or the fork flow is modified. Transcription is purely additive, running in the CLI process after the daemon exits.

## Implementation Approach

**Strategy: run transcription in the CLI (`/record stop`) process, not the daemon.** The daemon keeps its narrow responsibility (capture audio, flush Opus, exit). The CLI process, which already has a live terminal for stdout, takes over after `stop_session()` returns and does all transcription work synchronously.

**Why this split:**
- The daemon has stdout/stderr redirected to `recorder.log` (see `recorder/src/main.rs:51–61`). It cannot render a progress bar to the user.
- The CLI process is the user's terminal. Progress bars, prompts, and colored output all work naturally.
- If transcription fails or the user hits Ctrl-C, the audio file is already on disk — no coordinated cleanup needed.
- Decoupling simplifies testing: transcription is a pure function of `(opus_path, model_path) → transcript.json`, callable from unit tests without a running daemon.

**Module layout:**

```
recorder/src/
├── main.rs                    # cmd_stop() extended to call transcription
├── transcription/
│   ├── mod.rs                 # public entry: run_on_session(session_dir) -> Result<()>
│   ├── model.rs               # locate/download/verify ggml-small.en.bin
│   ├── decode.rs              # opus file -> two 48kHz mono f32 buffers (symphonia)
│   ├── resample.rs            # 48kHz -> 16kHz mono (rubato)
│   ├── whisper.rs             # whisper-rs wrapper, per-channel transcription
│   ├── merge.rs               # interleave segments from both channels by start time
│   ├── output.rs              # write transcript.json + transcription.log
│   └── progress.rs            # indicatif progress bar + ETA
```

**Dependency additions** (`recorder/Cargo.toml`):

```toml
# Audio decode + resample
symphonia = { version = "0.5", default-features = false, features = ["ogg", "opus"] }
rubato = "0.16"

# Whisper (GPU per target)
# (platform-specific block below)

# Model download + verification
ureq = { version = "2", features = ["tls"] }
sha2 = "0.10"
indicatif = "0.17"

# JSON output (serde/serde_json already present)

[target.'cfg(target_os = "macos")'.dependencies]
screencapturekit = "1.5"    # existing
whisper-rs = { version = "0.13", default-features = false, features = ["metal"] }

[target.'cfg(target_os = "windows")'.dependencies]
whisper-rs = { version = "0.13", default-features = false, features = ["vulkan"] }
```

No new `build.rs` — `whisper-rs` builds its bundled `whisper.cpp` via its own `build.rs`.

## Phase 1: Model Management

### Overview

Locate, download, and verify the `ggml-small.en.bin` file. This phase has no dependency on the audio pipeline and can be developed and tested in isolation.

### Changes Required

#### 1. New module: model discovery + fetch

**File:** `recorder/src/transcription/model.rs`

**Changes:** New module.

```rust
use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

pub const MODEL_FILENAME: &str = "ggml-small.en.bin";
pub const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin";
/// SHA256 of ggml-small.en.bin as published by ggerganov/whisper.cpp.
/// Pinned to detect corruption and partial downloads.
pub const MODEL_SHA256_HEX: &str =
    "1be3a9b2df72efb7d9b5d1e07a3b2a9a6c7e5d4c3b2a1f0e9d8c7b6a5f4e3d2c"; // TODO: replace with real hash at implementation time

fn models_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("no home directory")?;
    let dir = home.join(".domino").join("models");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(dir)
}

/// Returns a path to a verified model file, downloading if necessary.
pub fn ensure_model_available() -> Result<PathBuf> {
    let path = models_dir()?.join(MODEL_FILENAME);
    if path.exists() {
        match verify_sha256(&path, MODEL_SHA256_HEX) {
            Ok(true) => return Ok(path),
            Ok(false) => {
                tracing::warn!("model SHA mismatch; re-downloading");
                fs::remove_file(&path).ok();
            }
            Err(e) => {
                tracing::warn!(?e, "could not verify model; re-downloading");
                fs::remove_file(&path).ok();
            }
        }
    }
    download_with_progress(MODEL_URL, &path)?;
    if !verify_sha256(&path, MODEL_SHA256_HEX)? {
        fs::remove_file(&path).ok();
        bail!("downloaded model failed SHA256 verification");
    }
    Ok(path)
}

fn verify_sha256(path: &Path, expected_hex: &str) -> Result<bool> {
    let mut f = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    let got = hex::encode(hasher.finalize());
    Ok(got.eq_ignore_ascii_case(expected_hex))
}

fn download_with_progress(url: &str, dest: &Path) -> Result<()> {
    // Support resume via Range if a .part file already exists from an
    // interrupted download.
    let part_path = dest.with_extension("bin.part");
    let existing = fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0);

    let mut req = ureq::get(url);
    if existing > 0 {
        req = req.set("Range", &format!("bytes={}-", existing));
    }
    let resp = req.call().context("model download request failed")?;
    let total: u64 = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .map(|n: u64| n + existing)
        .unwrap_or(0);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "downloading ggml-small.en.bin [{bar:40}] {bytes}/{total_bytes} ({eta})",
        )?.progress_chars("#>-"),
    );
    pb.set_position(existing);

    let mut reader = resp.into_reader();
    let mut out = BufWriter::new(
        fs::OpenOptions::new()
            .create(true).append(existing > 0).write(true)
            .open(&part_path)?,
    );
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        out.write_all(&buf[..n])?;
        pb.inc(n as u64);
    }
    out.flush()?;
    drop(out);
    fs::rename(&part_path, dest)?;
    pb.finish_with_message("model ready");
    Ok(())
}
```

#### 2. Module declaration

**File:** `recorder/src/main.rs`

**Changes:** add `mod transcription;` near the top, next to existing `mod audio;` line.

```rust
mod audio;
mod cli;
mod session;
mod signals;
mod transcription;   // NEW
```

#### 3. Cargo.toml additions

**File:** `recorder/Cargo.toml`

**Changes:** add `ureq`, `sha2`, `indicatif`, `hex` to `[dependencies]`.

### Success Criteria

#### Automated Verification:
- [ ] `cargo build` succeeds on macOS: `cd recorder && cargo build`
- [ ] `cargo test transcription::model` passes (unit tests for `verify_sha256` and a mocked download happy path)
- [ ] `cargo clippy -- -D warnings` is clean for the new module
- [ ] Calling `ensure_model_available()` twice in a row is a no-op on the second call (no re-download)
- [ ] Corrupting the model file (truncate 1 byte) triggers re-download on next call

#### Manual Verification:
- [ ] Delete `~/.domino/models/ggml-small.en.bin`, run a smoke harness that calls `ensure_model_available()`, observe progress bar filling to 100%, verify file is ~465 MB and SHA256 matches
- [ ] Kill the download at ~50%, re-run, verify it resumes from where it stopped (no full re-download)
- [ ] Run on a machine with no network, with no existing model — verify a clear error message is printed

**Implementation Note:** Pause here for manual confirmation before moving to Phase 2.

---

## Phase 2: Audio Decode + Resample

### Overview

Read `meeting.opus` and produce two 16 kHz mono `Vec<f32>` buffers — one per channel. This phase is standalone: given any stereo `.opus` file, it outputs two resampled mono streams, verifiable in isolation.

### Changes Required

#### 1. Opus decode → stereo 48 kHz f32

**File:** `recorder/src/transcription/decode.rs`

**Changes:** New module.

```rust
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_OPUS};
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Returns `(left_48k, right_48k, duration_sec)` where each channel is mono f32 at 48kHz.
pub fn decode_stereo_opus(path: &Path) -> Result<(Vec<f32>, Vec<f32>, f64)> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("opus");
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
    let mut format = probed.format;

    let track = format
        .tracks().iter().find(|t| t.codec_params.codec == CODEC_TYPE_OPUS)
        .context("no Opus track in file")?;
    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(48_000);
    if sample_rate != 48_000 {
        bail!("expected 48kHz source, got {sample_rate}");
    }
    let n_channels = track.codec_params.channels
        .map(|c| c.count()).unwrap_or(0);
    if n_channels != 2 {
        bail!("expected stereo source, got {n_channels} channels");
    }

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;

    let mut left: Vec<f32> = Vec::new();
    let mut right: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };
        if packet.track_id() != track_id { continue; }

        match decoder.decode(&packet) {
            Ok(AudioBufferRef::F32(buf)) => {
                left.extend_from_slice(buf.chan(0));
                right.extend_from_slice(buf.chan(1));
            }
            Ok(other) => {
                // Convert to f32
                let mut tmp = other.make_equivalent::<f32>();
                other.convert(&mut tmp);
                left.extend_from_slice(tmp.chan(0));
                right.extend_from_slice(tmp.chan(1));
            }
            Err(SymError::DecodeError(_)) => continue, // skip bad packet, keep going
            Err(e) => return Err(e.into()),
        }
    }

    let duration = left.len() as f64 / sample_rate as f64;
    Ok((left, right, duration))
}
```

#### 2. Resampler 48 → 16 kHz

**File:** `recorder/src/transcription/resample.rs`

**Changes:** New module.

```rust
use anyhow::{Context, Result};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

/// Resample a mono f32 buffer from `from_hz` to `to_hz`.
pub fn resample_mono(input: &[f32], from_hz: u32, to_hz: u32) -> Result<Vec<f32>> {
    if from_hz == to_hz {
        return Ok(input.to_vec());
    }
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 160,
        window: WindowFunction::BlackmanHarris2,
    };
    // 1024 input frames per chunk; rubato handles the tail internally.
    let mut resampler = SincFixedIn::<f32>::new(
        to_hz as f64 / from_hz as f64,
        2.0,
        params,
        1024,
        1,
    ).context("failed to build resampler")?;

    let mut out = Vec::with_capacity(
        (input.len() as f64 * to_hz as f64 / from_hz as f64) as usize + 1024,
    );
    let chunk = resampler.input_frames_next();
    let mut pos = 0;
    let input_vec = vec![input.to_vec()];
    while pos + chunk <= input.len() {
        let slice = &input_vec[0][pos..pos + chunk];
        let out_frames = resampler.process(&[slice], None)?;
        out.extend_from_slice(&out_frames[0]);
        pos += chunk;
    }
    // Flush tail with zero-padding.
    if pos < input.len() {
        let mut tail = input[pos..].to_vec();
        tail.resize(chunk, 0.0);
        let out_frames = resampler.process(&[tail.as_slice()], None)?;
        out.extend_from_slice(&out_frames[0]);
    }
    Ok(out)
}
```

### Success Criteria

#### Automated Verification:
- [ ] `cargo test transcription::decode` passes — test asserts that decoding a 10-second stereo opus fixture produces two buffers of length ≈ 480_000 samples each
- [ ] `cargo test transcription::resample` passes — test asserts that resampling 48_000 samples of 48kHz silence to 16kHz yields ≈ 16_000 samples within ±10
- [ ] Round-trip test: encode a known sine wave with `audiopus` → write an Ogg Opus via `ogg` → decode via `decode_stereo_opus` → verify peak-correlation between input and decoded left channel > 0.95
- [ ] `cargo clippy -- -D warnings` clean

#### Manual Verification:
- [ ] Record a 30-second session (user speaks + YouTube plays), then write a dev harness (`cargo test --features dev-harness`) that decodes + resamples and dumps `left_16k.wav` and `right_16k.wav`. Open both in Audacity and confirm: `left_16k.wav` has user's voice only, `right_16k.wav` has YouTube audio only, both at 16 kHz.
- [ ] Confirm decoded `duration_sec` matches `ffprobe meeting.opus | grep Duration` within 50 ms.

**Implementation Note:** Pause here for manual confirmation before moving to Phase 3.

---

## Phase 3: Whisper Integration

### Overview

Load `ggml-small.en` into a `WhisperContext`, run inference on a single mono 16 kHz f32 buffer, return typed segments. Wrap with per-channel invocation (two calls: Left → "You", Right → "Meeting") that shares the same context for memory efficiency.

### Changes Required

#### 1. Whisper wrapper

**File:** `recorder/src/transcription/whisper.rs`

**Changes:** New module.

```rust
use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug, Clone)]
pub struct Segment {
    pub start_sec: f64,
    pub end_sec: f64,
    pub speaker: Speaker,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker { You, Meeting }

impl Speaker {
    pub fn as_str(&self) -> &'static str {
        match self { Speaker::You => "You", Speaker::Meeting => "Meeting" }
    }
}

pub struct Transcriber {
    ctx: WhisperContext,
    pub accelerator: &'static str,
}

impl Transcriber {
    pub fn load(model_path: &Path) -> Result<Self> {
        let mut cparams = WhisperContextParameters::default();
        cparams.use_gpu(true);  // honored only if a GPU feature is compiled in
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().context("non-utf8 model path")?,
            cparams,
        )?;
        let accelerator = if cfg!(feature = "metal") { "metal" }
            else if cfg!(feature = "vulkan") { "vulkan" }
            else { "cpu" };
        Ok(Self { ctx, accelerator })
    }

    /// Transcribe a single mono 16kHz f32 buffer. `speaker` is attached to
    /// every returned segment. Returns segments sorted by start time.
    pub fn transcribe(
        &self,
        samples_16k_mono: &[f32],
        speaker: Speaker,
    ) -> Result<Vec<Segment>> {
        let mut state = self.ctx.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_no_speech_thold(0.6);
        params.set_n_threads(
            std::thread::available_parallelism().map(|n| n.get() as i32).unwrap_or(4),
        );

        state.full(params, samples_16k_mono)?;

        let n = state.full_n_segments()?;
        let mut out = Vec::with_capacity(n as usize);
        for i in 0..n {
            let text = state.full_get_segment_text(i)?.trim().to_string();
            if text.is_empty() { continue; }
            // whisper timestamps are in 10ms units
            let t0 = state.full_get_segment_t0(i)? as f64 / 100.0;
            let t1 = state.full_get_segment_t1(i)? as f64 / 100.0;
            out.push(Segment { start_sec: t0, end_sec: t1, speaker, text });
        }
        Ok(out)
    }
}
```

### Success Criteria

#### Automated Verification:
- [ ] `cargo build` links against `whisper.cpp` on macOS with Metal feature, on Windows with Vulkan feature
- [ ] `cargo test transcription::whisper::test_model_loads` passes (loads `ggml-small.en.bin` if present; ignored via `#[ignore]` if not, so CI without the model still passes)
- [ ] `cargo test transcription::whisper::test_transcribes_known_sample --ignored` — feeds the bundled `jfk.wav` equivalent (we generate a 5-second synthetic speech WAV fixture from a known TTS) and asserts segment count ≥ 1 and text contains a known word
- [ ] Clippy clean

#### Manual Verification:
- [ ] On M2 Mac: transcribe a 60s fixture, observe tracing log reporting `accelerator="metal"`; wall-clock time < 5s
- [ ] On Windows 11 (Vulkan-capable GPU): same fixture, wall-clock time < 10s, `accelerator="vulkan"`
- [ ] On CPU-only fallback (disable GPU feature via env var): same fixture completes in < 30s

**Implementation Note:** Pause here for manual confirmation before moving to Phase 4.

---

## Phase 4: Merge + Output

### Overview

Combine segments from both channels into a single time-sorted list and serialize to `transcript.json`. Emit `transcription.log` alongside it.

### Changes Required

#### 1. Segment merge

**File:** `recorder/src/transcription/merge.rs`

```rust
use super::whisper::Segment;

/// Merge segment lists from both channels, sorted ascending by start time.
/// On exact tie, "You" sorts before "Meeting" to make output deterministic.
pub fn merge_segments(mut you: Vec<Segment>, mut meeting: Vec<Segment>) -> Vec<Segment> {
    let mut out = Vec::with_capacity(you.len() + meeting.len());
    out.append(&mut you);
    out.append(&mut meeting);
    out.sort_by(|a, b| {
        a.start_sec.partial_cmp(&b.start_sec)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                use super::whisper::Speaker::*;
                match (a.speaker, b.speaker) {
                    (You, Meeting) => std::cmp::Ordering::Less,
                    (Meeting, You) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            })
    });
    out
}
```

#### 2. JSON serialization

**File:** `recorder/src/transcription/output.rs`

```rust
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;
use super::whisper::Segment;

#[derive(Serialize)]
pub struct TranscriptFile<'a> {
    pub version: u32,
    pub audio_file: &'a str,
    pub duration_sec: f64,
    pub model: &'a str,
    pub model_sha256: &'a str,
    pub language: &'a str,
    pub transcribed_at: String,
    pub transcription_wall_sec: f64,
    pub accelerator: &'a str,
    pub segments: Vec<SerSegment<'a>>,
}

#[derive(Serialize)]
pub struct SerSegment<'a> {
    pub start: f64,
    pub end: f64,
    pub speaker: &'a str,
    pub text: &'a str,
}

pub fn write_transcript_json(
    path: &Path,
    audio_file: &str,
    duration_sec: f64,
    model_sha256: &str,
    wall_sec: f64,
    accelerator: &str,
    segments: &[Segment],
) -> Result<()> {
    let ser: Vec<SerSegment> = segments.iter().map(|s| SerSegment {
        start: s.start_sec,
        end: s.end_sec,
        speaker: s.speaker.as_str(),
        text: s.text.as_str(),
    }).collect();

    let file = TranscriptFile {
        version: 1,
        audio_file,
        duration_sec,
        model: "ggml-small.en",
        model_sha256,
        language: "en",
        transcribed_at: chrono::Local::now().to_rfc3339(),
        transcription_wall_sec: wall_sec,
        accelerator,
        segments: ser,
    };

    let pretty = serde_json::to_string_pretty(&file)?;
    fs::write(path, pretty).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
```

### Success Criteria

#### Automated Verification:
- [ ] `cargo test transcription::merge::test_interleaves_by_start` passes (construct segments with interleaved start times, verify sort order)
- [ ] `cargo test transcription::merge::test_tie_prefers_you` passes
- [ ] `cargo test transcription::output::test_roundtrip` passes — write a `TranscriptFile`, parse it back with `serde_json`, assert deep equality of segments
- [ ] The written JSON file validates against the schema documented in "Desired End State" (JSON schema assertion via `jsonschema` crate in test)

#### Manual Verification:
- [ ] Invoke the Phase 4 merge/output layer directly with two small mock segment lists (one for `"You"`, one for `"Meeting"`), write `transcript.json`, and spot-check that:
  - segment `start` times are monotonically non-decreasing
  - an exact timestamp tie sorts `"You"` before `"Meeting"`
  - speaker values are only `"You"` and `"Meeting"`
  - top-level metadata matches the v1 schema in "Desired End State"
  - the file is valid JSON and readable by `jq` / `serde_json`

Note: `/record stop` is **not** the Phase 4 verification surface. CLI-triggered transcription is wired in Phase 5; before that lands, a real recording should only be expected to produce `meeting.opus`.

**Implementation Note:** Pause here for manual confirmation before moving to Phase 5.

---

## Phase 5: Wire Into `/record stop`

### Overview

Glue the above into the single public entry point `transcription::run_on_session(session_dir)` and call it from `cmd_stop()`. Stream progress to stdout. Handle errors without losing the audio file.

### Changes Required

#### 1. Public entry point

**File:** `recorder/src/transcription/mod.rs`

```rust
mod decode;
mod merge;
pub mod model;
mod output;
mod progress;
mod resample;
mod whisper;

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;

pub struct RunOutcome {
    pub transcript_path: std::path::PathBuf,
    pub segment_count: usize,
    pub duration_sec: f64,
    pub wall_sec: f64,
    pub accelerator: &'static str,
}

/// Run the full transcription pipeline on a session directory.
/// Expects `<session_dir>/meeting.opus` to exist and be a finalized stereo Opus file.
pub fn run_on_session(session_dir: &Path) -> Result<RunOutcome> {
    let _log_guard = progress::init_log_file(&session_dir.join("transcription.log"))?;

    let opus_path = session_dir.join("meeting.opus");
    if !opus_path.exists() {
        anyhow::bail!("meeting.opus not found at {}", opus_path.display());
    }

    println!("Finalizing audio...");
    let t_all = Instant::now();

    // 1. Ensure model
    println!("Checking transcription model...");
    let model_path = model::ensure_model_available()
        .context("could not get ggml-small.en model")?;

    // 2. Decode
    let t = Instant::now();
    let (left_48k, right_48k, duration_sec) = decode::decode_stereo_opus(&opus_path)?;
    tracing::info!(
        decode_ms = t.elapsed().as_millis() as u64,
        samples = left_48k.len(),
        "opus decoded"
    );

    // 3. Resample
    let t = Instant::now();
    let left_16k = resample::resample_mono(&left_48k, 48_000, 16_000)?;
    let right_16k = resample::resample_mono(&right_48k, 48_000, 16_000)?;
    drop((left_48k, right_48k));
    tracing::info!(resample_ms = t.elapsed().as_millis() as u64, "resampled to 16kHz");

    // 4. Transcribe (sequential; one whisper context shared across two states)
    let tr = whisper::Transcriber::load(&model_path)?;
    let accelerator = tr.accelerator;

    let pb = progress::overall_bar(duration_sec);
    pb.set_message(format!(
        "transcribing with ggml-small.en ({accelerator})..."
    ));
    pb.set_position(0);

    let t = Instant::now();
    pb.set_message("transcribing mic channel");
    let you = tr.transcribe(&left_16k, whisper::Speaker::You)?;
    pb.set_position((duration_sec / 2.0 * 1000.0) as u64);

    pb.set_message("transcribing system channel");
    let meeting = tr.transcribe(&right_16k, whisper::Speaker::Meeting)?;
    pb.set_position((duration_sec * 1000.0) as u64);
    pb.finish_with_message("transcription complete");
    tracing::info!(whisper_ms = t.elapsed().as_millis() as u64, "whisper done");

    // 5. Merge + write
    let segments = merge::merge_segments(you, meeting);
    let segment_count = segments.len();
    let wall_sec = t_all.elapsed().as_secs_f64();

    let transcript_path = session_dir.join("transcript.json");
    output::write_transcript_json(
        &transcript_path,
        "meeting.opus",
        duration_sec,
        model::MODEL_SHA256_HEX,
        wall_sec,
        accelerator,
        &segments,
    )?;

    Ok(RunOutcome {
        transcript_path,
        segment_count,
        duration_sec,
        wall_sec,
        accelerator,
    })
}
```

#### 2. Progress + log plumbing

**File:** `recorder/src/transcription/progress.rs`

```rust
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::path::Path;
use tracing_subscriber::fmt::MakeWriter;

pub fn overall_bar(duration_sec: f64) -> ProgressBar {
    let total_ms = (duration_sec * 1000.0) as u64 * 2; // two passes
    let pb = ProgressBar::new(total_ms);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg:<40} [{bar:30}] {percent}% (ETA {eta})",
        ).unwrap().progress_chars("#>-"),
    );
    pb
}

pub struct LogGuard { _inner: tracing::subscriber::DefaultGuard }

/// Tee tracing output to `transcription.log` in addition to stderr.
pub fn init_log_file(path: &Path) -> Result<LogGuard> {
    let file = File::create(path)?;
    let sub = tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false)
        .finish();
    let guard = tracing::subscriber::set_default(sub);
    Ok(LogGuard { _inner: guard })
}
```

Note: the approach above scopes the log-file subscriber to the transcription call via `set_default`. The main binary's existing stderr `tracing_subscriber::fmt()` (`recorder/src/main.rs:19–25`) is re-entered after `run_on_session` returns.

#### 3. Wire into `cmd_stop`

**File:** `recorder/src/main.rs`

**Changes:** replace the body of `cmd_stop()` (currently `recorder/src/main.rs:142–155`):

```rust
fn cmd_stop() -> Result<()> {
    let info = session::stop_session()?;

    let opus_path = info.session_dir.join("meeting.opus");
    if !opus_path.exists() {
        println!("Session stopped: {} (no audio file produced)", info.session_dir.display());
        return Ok(());
    }

    let size_mb = std::fs::metadata(&opus_path)?.len() as f64 / (1024.0 * 1024.0);

    // Run transcription. Any failure leaves meeting.opus in place.
    match transcription::run_on_session(&info.session_dir) {
        Ok(outcome) => {
            println!("Saved:");
            println!("  {}     ({:.1} MB)", opus_path.display(), size_mb);
            println!(
                "  {}  ({} segments, {:.0}s wall)",
                outcome.transcript_path.display(),
                outcome.segment_count,
                outcome.wall_sec,
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("Transcription failed: {e:#}");
            eprintln!("Audio is preserved at: {}", opus_path.display());
            eprintln!(
                "Logs: {}",
                info.session_dir.join("transcription.log").display(),
            );
            std::process::exit(2);
        }
    }
}
```

### Success Criteria

#### Automated Verification:
- [ ] End-to-end integration test `tests/e2e_transcription.rs`:
  - Starts the recorder via the public library entry points (not the forked daemon — driven synchronously in-test)
  - Feeds a 10-second synthetic stereo audio fixture through the encoder and writes `meeting.opus`
  - Calls `transcription::run_on_session()` and asserts `transcript.json` is present, segments non-empty, speakers are labeled correctly
- [ ] `cargo test --test e2e_transcription` green
- [ ] `cargo build --release` produces a single binary < 15 MB on macOS, < 20 MB on Windows (whisper.cpp static link bloats this — if we bust the budget, revisit)

#### Manual Verification:
- [ ] Full smoke test on macOS: `/record start`, speak for 30s while playing a YouTube video, `/record stop`. Verify stdout shows progress bar, final output lists both `meeting.opus` and `transcript.json`, and `transcript.json` contains plausible "You" and "Meeting" segments.
- [ ] Ctrl-C during transcription: verify `meeting.opus` is intact, no corrupt `transcript.json` is written (either the file doesn't exist or it's the fully-written file from a completed previous run).
- [ ] Simulate missing model (delete `~/.domino/models/ggml-small.en.bin`), `/record stop` triggers on-demand download with progress bar, then continues transcription.
- [ ] Simulate corrupted model (truncate file), verify SHA mismatch triggers re-download.
- [ ] 60-minute recording: confirm wall-clock transcription time is < 5 minutes on M2 Mac with Metal.

**Implementation Note:** Full manual acceptance required before merging.

---

## Testing Strategy

### Unit Tests

- `transcription::model`: SHA verification (correct, wrong, truncated), resume-from-partial, URL rejected if non-HTTPS (security guard).
- `transcription::decode`: stereo 48 kHz asserted; mono or 44.1 kHz Opus files rejected with clear error; duration matches frame count.
- `transcription::resample`: 48→16 kHz ratio within ±0.01%; silence stays silence; known sine wave round-trips with correlation > 0.95.
- `transcription::merge`: ordering correctness, tie-breaking determinism, empty inputs.
- `transcription::output`: JSON schema conformance, round-trip equality.

### Integration Tests

- `tests/e2e_transcription.rs`: full pipeline on a 10s synthetic fixture (generated at test time from a bundled short TTS clip or a sine+speech mix stored in `tests/fixtures/`), no real model required if we stub `Transcriber` behind a trait in test builds. Alternative: mark the test `#[ignore]` and provide `cargo test --ignored` which downloads the real model once in CI cache.
- `tests/cmd_stop.rs`: drives `cmd_stop()` against a pre-populated session directory containing a real `meeting.opus`, asserts `transcript.json` appears and exit code is 0.

### Manual Testing Steps

1. Fresh install simulation: `rm -rf ~/.domino`, run a recording, verify model downloads on first stop, transcript emerges.
2. Warm cache: immediately record another 10s clip, stop — verify no re-download, transcription starts within 1s.
3. Long meeting: record 60 minutes of actual meeting-like content, stop, verify transcript quality is coherent and `transcription_wall_sec` < 300 on M2 Mac.
4. Edge: record 3 seconds only, stop. Verify transcription still produces a valid (possibly empty segment list) `transcript.json`, does not crash.
5. Edge: record with no microphone input (mute), only system audio. Verify `"You"` segments are absent or empty, `"Meeting"` segments are present.
6. Edge: record with no system audio (system capture unavailable, e.g., Screen Recording permission denied — right channel is silent per `recorder/src/main.rs:87–94`). Verify transcription produces `"You"` segments only, no crashes.

## Performance Considerations

- **Two sequential whisper passes** (one per channel). 60-minute recording on M2 Mac with Metal: ~2 minutes each, ~4 minutes total. Acceptable for v1.
  - *Not optimizing for v1*: parallelizing the two passes on a single GPU usually doesn't help (whisper saturates the accelerator); parallelizing on CPU would help but adds threading complexity. Revisit if users complain.
- **Memory peak**: 60-minute 48 kHz f32 stereo = ~1.4 GB held until resample; after resample and drop, ~500 MB for the two 16 kHz buffers. Acceptable; whisper itself holds another ~1 GB for the small.en model on GPU. Total working set ~2 GB during transcription.
  - *Mitigation path if needed later*: stream decode + resample in chunks, accumulate only the 16 kHz buffer. Not worth the code until a user reports OOM.
- **GPU feature flags at compile time** mean we must produce distinct builds per accelerator. This is already the case for the macOS/Windows split. No additional matrix expansion needed.

## Migration Notes

No migration — this is purely additive. Existing `meeting.opus` files from before this change can be transcribed by a hidden dev-only subcommand (`domino-recorder --dev-transcribe <session_dir>`) which is not exposed in production CLI help. Not part of the v1 ship.

## References

- Project vision & scope: `so_far.md`
- Prior capture implementation plan: `thoughts/shared/plans/2026-04-15-domino-v1-macos-audio-capture.md`
- Current daemon entry point: `recorder/src/main.rs:37–140`
- Current stop flow: `recorder/src/main.rs:142–155`, `recorder/src/session.rs:136–167`
- Existing stereo encoder that produces our input file: `recorder/src/audio/encoder.rs:87–248`
- `whisper.cpp` model registry: https://huggingface.co/ggerganov/whisper.cpp
- `whisper-rs` API: https://docs.rs/whisper-rs/latest/whisper_rs/
- `symphonia` Opus decoder: https://docs.rs/symphonia/latest/symphonia/
- `rubato` resampler: https://docs.rs/rubato/latest/rubato/
