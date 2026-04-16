# Domino v1 — macOS Audio Capture Implementation Plan

## Overview

Build v1 of Domino: a Claude Code plugin that captures both the user's microphone and macOS system audio during a meeting, interleaves them into a single stereo Opus file (mic-left / system-right), and stores it locally. macOS-only. Capture and storage only — no transcription, no analysis. Windows follows in v2.

The deliverable is a working end-to-end loop: `/record start` → meeting happens → `/record stop` → a single `meeting.opus` file appears on disk with both channels intact.

## Current State Analysis

The repo at `/Users/nitin/domino` is empty aside from `so_far.md` (project context, decisions, scope) and this plan. There is no code, no build system, no CI, no plugin manifest. v1 is a greenfield build.

All architectural decisions are already locked in `so_far.md`. Recap of the ones this plan depends on:

- **Language:** Rust for the recorder helper (cross-platform audio, small self-contained binaries).
- **macOS APIs:** `AVAudioEngine` / CoreAudio via `cpal` for microphone; `ScreenCaptureKit` (macOS 13+) via `screencapturekit-rs` for system audio. No driver install.
- **Output format:** Single stereo Opus file, 48 kHz, ~64 kbps, mic on left channel, system on right.
- **Plugin target:** Claude Code only for v1.
- **Signing:** Ad-hoc signed for v1 (no paid Apple Developer ID yet). Gatekeeper warning on first run is acceptable for early dogfood.
- **Distribution:** GitHub release + install script.
- **Storage:** `~/.domino/recordings/<session-id>/meeting.opus`.
- **Session model:** One recording at a time. A second `/record start` while one is running fails with a clear error.
- **Repo layout:** Monorepo — `recorder/` (Rust crate) + `plugin/` (Claude Code plugin, TS/JS).

### Key Discoveries:

- `cpal` gives us a unified cross-platform audio capture API for microphones and is already idiomatic for Rust audio work. Windows loopback is a free bonus when we get there in v2.
- `screencapturekit-rs` exposes audio-only `SCStream` configuration, so we never have to negotiate video frames.
- `audiopus` (Rust bindings to libopus) is the mature path for inline Opus encoding. `ogg` crate handles the Ogg container muxing.
- macOS TCC permissions (Microphone, Screen Recording) are granted **per parent process**. For a Claude Code user, that parent is their terminal emulator (Terminal.app, iTerm, Warp, the VS Code integrated terminal, etc.). We can't fix this; we can only detect it and explain it.
- Claude Code plugin system supports slash commands via a plugin manifest + a command handler (TypeScript / JS). The plugin shells out to our Rust binary rather than trying to do audio work from Node.

## Desired End State

A user on macOS 13+ can:

1. Install the plugin with a single shell one-liner.
2. Open Claude Code in their preferred terminal.
3. Run `/record start`. First time ever: be told, plainly and specifically, which two macOS permissions to grant and where to click. On every subsequent run: recording begins silently within 500 ms.
4. Close the terminal window, attend the meeting, come back.
5. Run `/record stop`. See a confirmation like `Saved: ~/.domino/recordings/2026-04-15-1423/meeting.opus (24.1 MB, 47m 12s)`.
6. Open the file in any Opus-compatible player (VLC, ffmpeg) and hear both their own voice (left channel) and the meeting (right channel) cleanly.

### Verification

- `ffprobe meeting.opus` reports: codec `opus`, 2 channels, 48000 Hz, duration matching actual meeting length ±1 s.
- `ffmpeg -i meeting.opus -map_channel 0.0.0 mic.wav` produces an audible mic-only track.
- `ffmpeg -i meeting.opus -map_channel 0.0.1 system.wav` produces an audible system-only track.
- Closing the terminal during recording does **not** stop the recording. The file is finalized only when `/record stop` is run (or on SIGTERM / system shutdown).
- A second `/record start` while a session is active exits non-zero with message `A recording is already in progress (session: 2026-04-15-1423, PID: 12345). Run /record stop first.`
- `/record doctor` on a fresh machine with no permissions granted prints a checklist of missing permissions with exact click-paths to System Settings.

## What We're NOT Doing

Explicitly out of scope for v1. Do not add these even if tempted:

- **Windows support.** v2.
- **Linux support.** No v2 commitment.
- **Transcription.** The file sits on disk. A later phase reads it.
- **Diarization.** Same.
- **LLM synthesis of meeting content → proactive task generation.** Same.
- **Plan approval UX in the assistant.** Same.
- **Echo cancellation.** The stereo split makes it *possible* later; we don't implement it in v1.
- **Streaming upload / cloud storage.** Local only.
- **Automatic cleanup of old recordings.** Manual `rm` for now.
- **Multiple concurrent sessions.** One at a time, enforced by PID file.
- **Configurable bitrate / sample rate.** Hardcoded to 48 kHz stereo @ 64 kbps. Flags can come later.
- **Pause / resume.** Start and stop only.
- **A `/record doctor --fix` that opens System Settings automatically.** Print the path and let the user click. (macOS exposes `x-apple.systempreferences:` URLs but behavior varies across OS versions; not worth the flakiness for v1.)
- **Notarization.** Ad-hoc signed. Users right-click → Open the first time if Gatekeeper complains.
- **A paid Apple Developer ID.** Deferred until public launch.
- **Codex plugin wrapper.** Claude Code only in v1.
- **Telemetry / usage analytics.** None.
- **A consent/disclaimer UI.** A single-line banner on first run is enough for dogfood. Real legal review before any public release.
- **Device selection (which mic, which output).** Always default input device for mic; always default output device for system audio capture.
- **Format output options (WAV, MP3, FLAC).** Opus only.

## Implementation Approach

The work splits cleanly into two mostly-independent tracks that converge at the end:

- **Track A (Rust recorder):** a self-contained CLI binary that captures, encodes, and writes the file. All the hard platform-specific work lives here. We build it test-first where we can and manually verify on a real Mac for what we can't.
- **Track B (Plugin wrapper):** a thin Claude Code plugin (TypeScript) that exposes `/record` slash commands and shells out to the Rust binary. This is mostly glue; the interesting logic is the first-run permissions flow and the session-lifecycle UX.

We build Track A first, get it rock-solid as a standalone CLI, then wrap it. This sequencing means we never debug "is this a plugin bug or a recorder bug" — the recorder is proven by the time we touch the plugin.

Phases are ordered by risk: microphone first (easy, builds the skeleton), then system audio (the hard part), then stereo muxing, then lifecycle, then the plugin wrapper, then distribution.

---

## Phase 1: Monorepo Bootstrap & Rust Recorder Skeleton

### Overview

Establish the repo layout, toolchain, CI, and a recorder binary that does nothing yet but parses CLI args, resolves a session directory, writes a PID file, and exits cleanly on SIGTERM. No audio yet. This phase exists so every later phase has a stable scaffold to grow into.

### Changes Required:

#### 1. Repo structure

**Files**: (new, at repo root)

```
/Users/nitin/domino/
├── recorder/                     # Rust crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs               # CLI entry
│   │   ├── cli.rs                # clap argument parsing
│   │   ├── session.rs            # session dir, PID file
│   │   └── signals.rs            # SIGTERM handling
│   └── tests/
├── plugin/                       # Claude Code plugin (populated in Phase 5)
│   └── .gitkeep
├── scripts/
│   └── install.sh                # populated in Phase 6
├── .github/
│   └── workflows/
│       └── ci.yml
├── .gitignore
├── README.md                     # minimal: "see so_far.md"
├── so_far.md                     # already exists
└── thoughts/                     # already exists
```

#### 2. `recorder/Cargo.toml`

**File**: `recorder/Cargo.toml`
**Changes**: create with MSRV pinned to Rust 1.75.

```toml
[package]
name = "domino-recorder"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[[bin]]
name = "domino-recorder"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
anyhow = "1"
thiserror = "1"
chrono = { version = "0.4", default-features = false, features = ["clock"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "5"                # for ~/.domino resolution
signal-hook = "0.3"       # portable SIGTERM handling
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[target.'cfg(target_os = "macos")'.dependencies]
# populated in Phase 2 (cpal) and Phase 3 (screencapturekit)

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "symbols"
```

#### 3. CLI surface

**File**: `recorder/src/cli.rs`
**Changes**: define the full v1 CLI up front even though most subcommands are stubs.

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "domino-recorder", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start a recording session. Writes session info to stdout as JSON.
    Start {
        /// Override default output directory (~/.domino/recordings).
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
    /// Stop the currently active recording session.
    Stop,
    /// Print active session info as JSON, or "{}" if none.
    Status,
    /// Print diagnostic info about permissions, devices, OS version.
    Doctor,
}
```

#### 4. Session directory + PID file

**File**: `recorder/src/session.rs`
**Changes**: resolve session paths, manage PID file atomically.

```rust
// Session dir: ~/.domino/recordings/<YYYY-MM-DD-HHMM>/
// Active PID file: ~/.domino/current.pid (contains JSON: {pid, session_dir, started_at})
// start(): fail if current.pid exists and process is alive; create session dir; write pid file
// stop(): read pid file, SIGTERM the process, wait up to 5s for it to clean up, then delete pid file
// status(): read pid file, check liveness, return session info or empty
```

The PID file is the single source of truth for "is a session running." We use `fs2::FileExt::try_lock_exclusive` for atomic creation to prevent two `start` invocations racing. We check process liveness via `kill(pid, 0)` to detect stale PID files from crashes.

#### 5. Signal handling

**File**: `recorder/src/signals.rs`
**Changes**: register SIGTERM/SIGINT handlers that set an atomic `shutdown` flag. Capture loops (added in Phase 2+) poll this flag and exit cleanly.

#### 6. GitHub Actions CI

**File**: `.github/workflows/ci.yml`
**Changes**: run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` on `macos-14` runner.

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --manifest-path recorder/Cargo.toml --check
      - run: cargo clippy --manifest-path recorder/Cargo.toml -- -D warnings
      - run: cargo test --manifest-path recorder/Cargo.toml
```

### Success Criteria:

#### Automated Verification:
- [ ] `cd recorder && cargo build --release` succeeds on macOS 14.
- [ ] `cargo clippy -- -D warnings` passes with no warnings.
- [ ] `cargo test` passes unit tests for `session.rs` (PID file creation, stale-detection, concurrent-start rejection).
- [ ] Running `./target/release/domino-recorder start` creates `~/.domino/current.pid` and a session directory.
- [ ] Running `./target/release/domino-recorder status` while the recorder is running prints session JSON; prints `{}` after stop.
- [ ] Running `./target/release/domino-recorder start` a second time while one is active exits with code 1 and a clear error.
- [ ] Running `./target/release/domino-recorder stop` cleanly terminates the running process and deletes the PID file.
- [ ] CI green on GitHub Actions.

#### Manual Verification:
- [ ] Repo layout looks clean and navigable.
- [ ] Sending SIGKILL to a running recorder leaves a stale PID file, and the next `start` detects it and proceeds (not stuck).
- [ ] `~/.domino/` is created with correct permissions (0700) on first run.

**Implementation Note**: After Phase 1 is green, pause for confirmation that the scaffold is correct before starting audio work.

---

## Phase 2: Microphone Capture → Mono Opus File

### Overview

Add real audio: capture from the default input device via `cpal`, encode inline to Opus, mux into an Ogg container, and write to `<session>/meeting.opus`. Mono for now — the system-audio channel comes in Phase 3. First manually-triggerable TCC prompt for Microphone appears here.

### Changes Required:

#### 1. New dependencies

**File**: `recorder/Cargo.toml`
**Changes**: add macOS audio + encoding deps.

```toml
[target.'cfg(target_os = "macos")'.dependencies]
cpal = "0.15"
audiopus = "0.3"             # Rust bindings to libopus
ogg = "0.9"                  # Ogg container muxing
ringbuf = "0.4"              # lock-free SPSC ring buffer between capture callback and encoder
```

libopus ships as a static lib via `audiopus`; no system dependency. Verify at build time.

#### 2. Audio pipeline module

**File**: `recorder/src/audio/mod.rs`, `recorder/src/audio/mic.rs`, `recorder/src/audio/encoder.rs`

**Architecture**:

```
[CoreAudio mic callback] → [ringbuf (lock-free)] → [encoder thread] → [Ogg/Opus file]
```

The capture callback must be real-time-safe (no allocs, no locks, no file I/O). It pushes f32 samples into a ring buffer. A dedicated encoder thread drains the ring buffer in 20 ms frames (960 samples @ 48 kHz), encodes each frame to Opus, and writes Ogg pages to the output file.

**`mic.rs`** — `start_mic_capture(sample_tx: Producer<f32>) -> Result<cpal::Stream>`:
- Get default host, default input device.
- Request a config of 48 kHz, 1 channel, f32 samples. If the device doesn't support this, pick the closest supported config and resample on the fly (rubato crate, added if needed).
- On each callback, push samples into `sample_tx`. If the ring buffer is full (encoder falling behind), drop samples and increment a dropped-samples counter logged via `tracing`.

**`encoder.rs`** — `spawn_encoder_thread(sample_rx, output_path) -> Result<JoinHandle>`:
- Open output file, write Ogg stream header with Opus-specific metadata (`OpusHead`, `OpusTags`).
- Loop: pull 960 samples, encode with `audiopus::Encoder` configured for VOIP at 32 kbps mono, write Ogg page.
- On shutdown signal: flush remaining samples, finalize the Ogg stream with the EOS flag, fsync, close file.

#### 3. Wire `start` to launch capture

**File**: `recorder/src/main.rs`
**Changes**: on `Command::Start`:
1. Acquire session lock (Phase 1).
2. Daemonize (fork + setsid + detach stdio). Use `daemonize` crate or equivalent. The Rust binary must survive its parent terminal closing. This is critical UX.
3. Redirect logs to `<session>/recorder.log`.
4. Start mic capture.
5. Start encoder thread.
6. Block on shutdown signal; on receipt, stop cpal stream, signal encoder to flush, join, exit 0.

#### 4. TCC permission handling

The first cpal input stream creation will trigger the TCC Microphone prompt automatically. If permission is denied, cpal returns a `BuildStreamError`. We catch this specifically and write a JSON error to stderr that the plugin (Phase 5) can parse:

```json
{"error": "permission_denied", "permission": "microphone", "message": "..."}
```

We do not try to preemptively detect missing Microphone permission — calling `AVCaptureDevice.requestAccess(for: .audio)` requires Objective-C FFI for no real benefit. Failing at stream-create time is the standard Apple pattern.

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test --test mic_smoke` runs a 2-second synthetic-input capture using cpal's `Device::default_input_config` on a CI runner (macOS 14 with virtual mic). File exists and is a valid Ogg Opus stream per `ffprobe`.
- [ ] Unit test: encoder handles exactly-one-frame input (960 samples) and produces a valid single-page Ogg file.
- [ ] Unit test: encoder handles partial-final-frame (e.g., 500 samples) by zero-padding before the final encode.
- [ ] Clippy + fmt still clean.

#### Manual Verification:
- [ ] On first `start` invocation on a fresh Mac account, macOS shows the Microphone permission prompt naming the parent terminal app.
- [ ] After granting permission and running `start` again, a file appears at `~/.domino/recordings/<session>/meeting.opus` while recording.
- [ ] Speaking into the mic for 30 seconds, then running `stop`, produces a playable file in VLC with audible speech.
- [ ] `ffprobe meeting.opus` reports codec=opus, channels=1, sample_rate=48000, duration≈30s.
- [ ] Closing the terminal hosting the recorder **does not** stop the recording — verified by `ps aux | grep domino-recorder` after closing.
- [ ] Recording for 5 minutes produces a reasonably-sized file (~1 MB) with no dropped-samples warnings in the log.

**Implementation Note**: Pause here. Mic-only recording is a useful standalone milestone, and confirming the lifecycle (daemonize, survive terminal close, clean stop) works before adding system audio saves debugging two unknowns at once.

---

## Phase 3: System Audio Capture via ScreenCaptureKit

### Overview

Add the second audio source: what the Mac is playing out of its default output device. This is the hardest single piece of the project. Uses `ScreenCaptureKit` in audio-only mode. Triggers the Screen Recording TCC prompt the first time.

### Changes Required:

#### 1. New dependency

**File**: `recorder/Cargo.toml`

```toml
[target.'cfg(target_os = "macos")'.dependencies]
screencapturekit = "0.3"     # Rust bindings to ScreenCaptureKit
# or, if that crate is insufficient, fall back to:
# objc2 = "0.5"
# objc2-screen-capture-kit = "0.2"
```

Evaluate `screencapturekit` crate first. If its audio-only stream support is mature, use it. If not, write a thin Objective-C FFI shim using `objc2` directly. Budget a half day to decide; do not spend more than that before picking a path.

#### 2. System audio capture module

**File**: `recorder/src/audio/system.rs`

**API**: `start_system_capture(sample_tx: Producer<f32>) -> Result<SystemCaptureHandle>`.

**Implementation sketch**:
1. Call `SCShareableContent.current` to get a list of displays. Pick the main display (we only need *a* display to attach the audio-only stream to — ScreenCaptureKit's API requires one).
2. Build an `SCStreamConfiguration` with:
   - `capturesAudio = true`
   - `excludesCurrentProcessAudio = true` (so we don't record our own output if the user's assistant happens to make sound)
   - `sampleRate = 48000`
   - `channelCount = 2` (ScreenCaptureKit gives stereo; we'll down-mix to mono for the right channel of the output file — see design note below)
   - Minimal video config (1x1 frame, lowest frame rate allowed) — we ignore video frames in the delegate. Some SCK versions allow audio-only; if so, use it.
3. Build an `SCContentFilter` including the chosen display.
4. Create an `SCStream` and an audio sample delegate.
5. Start the stream.
6. The audio delegate receives `CMSampleBuffer` objects. Extract PCM samples, down-mix stereo to mono (simple L+R/2), push to `sample_tx`.

**Design note on down-mixing**: ScreenCaptureKit delivers the system mix as stereo. Our output file's *right channel* is "system audio." We down-mix SCK's stereo to mono before pushing it into the right output channel. We lose stereo separation within the meeting audio (everyone's voice comes through centered). For v1's goal — feeding transcription — this is fine. Per-speaker stereo was never achievable from a system tap anyway; that's what diarization is for.

#### 3. Stereo muxing

**File**: `recorder/src/audio/encoder.rs`
**Changes**: encoder now takes *two* ring buffer consumers (mic + system) and interleaves them into a stereo buffer per frame.

- Pull 960 mono samples from mic ring buffer → fills left half of stereo frame.
- Pull 960 mono samples from system ring buffer → fills right half.
- Interleave as `[L0, R0, L1, R1, ...]` (Opus convention).
- Encode with `audiopus::Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Voip)` at 64 kbps.
- Write Ogg page.

**Synchronization**: mic and system streams run on independent clocks. If one ring buffer is consistently ahead of the other, we get drift. Mitigations:
- Use capture timestamps (`CMSampleBuffer.presentationTimeStamp` for SCK; cpal's stream timestamp for mic) to align frames at encode time.
- If a buffer is behind, insert a silent frame on that channel. If ahead, drop a frame.
- Log drift metrics every 30 seconds via `tracing`.

A small drift (<100 ms) across a meeting is acceptable for v1. Large drift (>500 ms) is a bug and should be logged as an error.

#### 4. Graceful degradation

If system audio capture fails at start (e.g., Screen Recording permission denied, SCK not available on this OS), log a warning and proceed with mic-only (right channel is silence). This is better than failing the whole recording. Status JSON includes a `system_audio_active: false` flag so the plugin can surface this to the user.

### Success Criteria:

#### Automated Verification:
- [ ] Unit test: stereo encoder produces valid 2-channel Ogg Opus output per ffprobe.
- [ ] Unit test: drift compensation inserts silent frames correctly when one stream is behind.
- [ ] Clippy + fmt clean.

#### Manual Verification:
- [ ] On first `start` after Phase 3 lands, Screen Recording TCC prompt appears. After granting and re-running, recording begins.
- [ ] Record for 30 seconds while playing a YouTube video in Safari and speaking into the mic simultaneously. Stop. Inspect output:
  - [ ] `ffprobe meeting.opus` reports 2 channels, 48 kHz.
  - [ ] `ffmpeg -i meeting.opus -map_channel 0.0.0 mic.wav && afplay mic.wav` → hear only the user's voice.
  - [ ] `ffmpeg -i meeting.opus -map_channel 0.0.1 system.wav && afplay system.wav` → hear only the YouTube audio.
  - [ ] Both tracks are audibly in sync when mixed.
- [ ] Record with Screen Recording permission revoked mid-session — recording continues with right channel silent (graceful degradation).
- [ ] Record a Zoom call to a test meeting with a colleague. Verify both sides are captured.
- [ ] Bluetooth headset connecting mid-recording does not crash the recorder.

**Implementation Note**: This phase is the hardest. Expect ScreenCaptureKit to have undocumented edge cases. Pause for manual confirmation that audio quality and sync are acceptable across several real meetings before proceeding.

---

## Phase 4: Lifecycle Polish + `doctor` Command

### Overview

Tighten the operational surface: control socket for reliable stop/status, doctor command for permission diagnosis, and format-change handling for sample-rate switches mid-recording (Bluetooth headset hot-swap is the most common trigger). Nothing new user-visible in the happy path; this phase exists to make the happy path reliable.

### Changes Required:

#### 1. Control socket

**Files**: `recorder/src/control.rs`
**Changes**: replace the "find PID, send SIGTERM" approach with a Unix domain socket at `~/.domino/control.sock`.

- Recorder daemon listens on the socket.
- Accepted protocol: line-delimited JSON commands (`{"cmd": "stop"}`, `{"cmd": "status"}`).
- `stop` triggers clean shutdown, returns `{"ok": true, "session_dir": "..."}`, then exits.
- `status` returns live stats: duration, bytes written, dropped samples, system_audio_active.

This gives us real-time status and a cleaner stop than signal-based shutdown (though we keep SIGTERM as a fallback).

#### 2. `doctor` command

**File**: `recorder/src/doctor.rs`
**Changes**: check and pretty-print:

- macOS version (must be 13.0+). Read via `sw_vers` or `NSProcessInfo`.
- Architecture (arm64 vs x64).
- TCC Microphone permission status for the current parent process. Detect by attempting a 0.1s cpal input stream and catching the error.
- TCC Screen Recording permission status. Detect by calling `CGPreflightScreenCaptureAccess()` via FFI.
- Default input device name and whether it's reachable.
- Default output device name.
- Whether a recording is currently active (checks control socket).
- Writable `~/.domino/` directory.

Output format (when all is well):

```
Domino Recorder — Health Check
  macOS:              14.4 (arm64)                 OK
  Microphone:         granted for iTerm.app        OK
  Screen Recording:   granted for iTerm.app        OK
  Default mic:        MacBook Pro Microphone       OK
  Default output:     MacBook Pro Speakers         OK
  Storage:            ~/.domino/ (rw)              OK
  Active session:     none                         OK
All checks passed. You're ready to record.
```

When broken:

```
  Screen Recording:   NOT GRANTED for iTerm.app    ✗
    → Open System Settings → Privacy & Security → Screen Recording
    → Enable iTerm.app
    → Restart iTerm, then re-run /record start
```

#### 3. Format-change handling

**File**: `recorder/src/audio/mic.rs`, `recorder/src/audio/system.rs`
**Changes**: register stream-error callbacks; on format change (e.g., Bluetooth headset connects and sample rate jumps from 48 kHz to 16 kHz), tear down and rebuild the stream with the new format, and insert a resampler before the ring buffer if the new rate differs from 48 kHz.

Use `rubato` crate for resampling. Cache the resampler; only rebuild when format actually changes.

#### 4. Log rotation

Recording log files (`<session>/recorder.log`) are bounded to 10 MB; rotate once.

### Success Criteria:

#### Automated Verification:
- [ ] Unit test: control socket handles `stop` and `status` commands correctly.
- [ ] Unit test: format-change callback rebuilds stream and resampler.
- [ ] `cargo test` green.

#### Manual Verification:
- [ ] `/record doctor` on a clean machine with no permissions reports both as missing with exact click-paths. Granting each and re-running shows them OK.
- [ ] `/record status` during a live recording returns current duration and byte count, updated in real time.
- [ ] Plugging in AirPods mid-recording does not corrupt the file; recording continues with the new input.
- [ ] Log file stays under 10 MB across a 2-hour recording.

---

## Phase 5: Claude Code Plugin Wrapper

### Overview

Build the thin TypeScript plugin that exposes `/record start`, `/record stop`, `/record status`, `/record doctor` as Claude Code slash commands. The plugin does no audio work — it locates the bundled Rust binary, invokes it, and renders the output nicely to the user.

### Changes Required:

#### 1. Plugin manifest

**File**: `plugin/plugin.json` (exact name and schema follow Claude Code plugin spec at implementation time)

```json
{
  "name": "domino",
  "version": "0.1.0",
  "description": "Record meeting audio (mic + system) locally for later analysis",
  "commands": [
    { "name": "record", "handler": "./dist/record.js" }
  ],
  "platforms": ["darwin-arm64", "darwin-x64"]
}
```

#### 2. Command handler

**File**: `plugin/src/record.ts`
**Changes**: implement a subcommand dispatcher.

```ts
// Subcommands: start, stop, status, doctor
// Locates binary at: <plugin-install-dir>/bin/domino-recorder-<arch>
// Spawns it with appropriate args, parses JSON output, renders for user.
// On "permission_denied" error from start, prints the first-run permission guidance.
```

**First-run permission flow**:

```
> /record start
Starting recording...
✗ Microphone access is required.

  Claude Code is running inside iTerm.app. macOS needs you to grant:
    1. Microphone       → System Settings → Privacy & Security → Microphone → enable "iTerm"
    2. Screen Recording → System Settings → Privacy & Security → Screen Recording → enable "iTerm"

  After granting, please restart iTerm.app, then run /record start again.
  (macOS requires the terminal to restart before new permissions take effect.)
```

The plugin detects the parent terminal by reading `$TERM_PROGRAM` or `$__CFBundleIdentifier` (set by most macOS terminals). Falls back to "your terminal" if unknown.

#### 3. Output rendering

- `/record start` success: one-line confirmation with session directory.
- `/record stop` success: confirmation with final file size and duration.
- `/record status` active: duration, bytes, any warnings.
- `/record status` idle: "No recording in progress."
- `/record doctor`: reprint the recorder's output verbatim (it's already formatted).

#### 4. Binary resolution

The plugin package ships with binaries for both macOS arches under `bin/`:

```
plugin/
├── bin/
│   ├── domino-recorder-darwin-arm64
│   └── domino-recorder-darwin-x64
├── dist/                        # compiled TS
├── src/
│   └── record.ts
├── plugin.json
├── package.json
└── tsconfig.json
```

At runtime, plugin detects arch via `process.arch` and spawns the matching binary. Both binaries are ad-hoc signed (`codesign --sign -` during release build).

#### 5. Build script

**File**: `plugin/package.json`
**Changes**: build scripts tie Rust + TS together.

```json
{
  "scripts": {
    "build:rust:arm64": "cd ../recorder && cargo build --release --target aarch64-apple-darwin && cp target/aarch64-apple-darwin/release/domino-recorder ../plugin/bin/domino-recorder-darwin-arm64",
    "build:rust:x64":   "cd ../recorder && cargo build --release --target x86_64-apple-darwin  && cp target/x86_64-apple-darwin/release/domino-recorder ../plugin/bin/domino-recorder-darwin-x64",
    "build:rust":       "npm run build:rust:arm64 && npm run build:rust:x64",
    "sign":             "codesign --sign - --force bin/domino-recorder-darwin-arm64 bin/domino-recorder-darwin-x64",
    "build:ts":         "tsc",
    "build":            "npm run build:rust && npm run sign && npm run build:ts"
  }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] `cd plugin && npm run build` completes without error on a macOS arm64 machine (builds arm64 binary; x64 binary requires cross-toolchain or x64 CI runner).
- [ ] `tsc` passes with strict mode.
- [ ] Plugin manifest validates against Claude Code's plugin schema (use the official validator if available).
- [ ] Unit test: command dispatcher correctly routes `/record start`, `/record stop`, etc.
- [ ] Unit test: permission-denied error parsing renders correct terminal-specific guidance.

#### Manual Verification:
- [ ] Install plugin into a local Claude Code instance manually.
- [ ] `/record start` on a fresh Mac account → see the first-run permission guidance with correct terminal name.
- [ ] After granting permissions and restarting the terminal, `/record start` succeeds silently.
- [ ] `/record stop` produces the confirmation line with session path.
- [ ] `/record status` during a recording shows live updates (re-run to see duration grow).
- [ ] `/record doctor` output is readable and actionable.
- [ ] End-to-end test: record a real 5-minute meeting via `/record start` → close terminal → reopen → `/record stop` → file is valid.

**Implementation Note**: This is the first phase where a user could actually use the product. Aggressive manual testing expected. Pause for approval before cutting a release.

---

## Phase 6: GitHub Release + Install Script

### Overview

Ship the thing. Automated release builds on tag push; one-line install for users.

### Changes Required:

#### 1. Release workflow

**File**: `.github/workflows/release.yml`
**Changes**: on `v*` tag push:
1. Build Rust recorder for both macOS targets (`aarch64-apple-darwin`, `x86_64-apple-darwin`) on `macos-14` runner.
2. Ad-hoc sign both binaries (`codesign --sign -`).
3. Build TS plugin (`npm run build:ts`).
4. Package into a tarball: `domino-<version>-darwin.tar.gz` containing `plugin/` directory with both binaries + compiled TS + manifest.
5. Upload as GitHub release asset.
6. Generate SHA256 checksum alongside.

#### 2. Install script

**File**: `scripts/install.sh`
**Changes**:

```bash
#!/usr/bin/env bash
set -euo pipefail

# 1. Verify macOS 13+
# 2. Detect arch (uname -m)
# 3. Fetch latest release from GitHub API
# 4. Download tarball + checksum, verify
# 5. Extract to ~/.claude/plugins/domino/ (or wherever Claude Code plugins live)
# 6. Print next-steps: "Open Claude Code and run /record doctor"
```

Install command for users:

```bash
curl -fsSL https://raw.githubusercontent.com/<org>/domino/main/scripts/install.sh | bash
```

The script prints every step it's doing, checks checksums, never runs sudo, and is under 100 lines.

#### 3. README

**File**: `README.md`
**Changes**: replace the stub with:
- One-paragraph project description.
- Install command.
- Four-line quickstart (`/record start`, do the thing, `/record stop`, find your file).
- Link to `so_far.md` for context/decisions.
- Known limitations (macOS 13+ only, ad-hoc signed means right-click-Open first time, etc.).
- Troubleshooting: "recording is silent" → `/record doctor`.

#### 4. Gatekeeper first-run guidance

Because we ship ad-hoc signed, macOS Gatekeeper will warn the user on first run. README includes:

> The first time you run `/record start`, macOS may block the recorder binary. To allow it:
> 1. Open System Settings → Privacy & Security
> 2. Scroll to the bottom. You'll see a message: "domino-recorder was blocked"
> 3. Click "Allow Anyway"
> 4. Re-run `/record start`
>
> You only need to do this once. We'll ship a notarized version when the project is ready for public release.

### Success Criteria:

#### Automated Verification:
- [ ] Tagging `v0.1.0` triggers the release workflow and produces a downloadable tarball.
- [ ] Tarball extracts cleanly; `plugin.json` validates; binaries are executable and ad-hoc signed (`codesign -dv` reports `Signature=adhoc`).
- [ ] SHA256 checksum matches.
- [ ] Install script syntax-checks with `shellcheck`.

#### Manual Verification:
- [ ] On a clean Mac (or fresh user account), `curl | bash` succeeds and installs the plugin.
- [ ] Opening Claude Code shows `/record` as a registered command.
- [ ] Full end-to-end: install → `/record start` → grant permissions → record a 10-minute real meeting → `/record stop` → verify the file.
- [ ] README's Gatekeeper guidance actually matches what macOS 14 shows on first blocked run (screenshots in my notes, not committed).

**Implementation Note**: Once this phase passes, v1 is done. Ship to 2-3 teammates, collect feedback, and only then start v2 (Windows) or v1.5 (transcription).

---

## Testing Strategy

### Unit Tests (Rust)

- `session.rs`: PID file atomicity, stale-PID detection, concurrent-start rejection.
- `audio/encoder.rs`: Ogg Opus output validity (stereo frame, partial final frame, silence padding), drift compensation.
- `control.rs`: command parsing, response shapes.
- `doctor.rs`: each check returns correct status given mocked inputs.

### Unit Tests (TS)

- `record.ts`: subcommand routing, error JSON parsing, permission-denied guidance rendering with different `$TERM_PROGRAM` values.

### Integration Tests (Rust)

- Spawn the recorder binary with a short max-duration flag (add for testing only), let it record against a synthetic input, verify output file structure.
- Two-start race test: launch two `start` invocations in parallel; exactly one succeeds.

### Manual Test Script

Checklist to run before each release:

1. Fresh macOS account (or scrubbed TCC: `tccutil reset All`).
2. `/record doctor` — confirm it reports both permissions missing.
3. `/record start` — grant Microphone. Re-run, grant Screen Recording. Restart terminal. Re-run — succeeds.
4. Play a YouTube video + speak for 60 seconds. `/record stop`.
5. Verify file: `ffprobe`, channel split, audible.
6. `/record start`, close terminal, wait 5 min, reopen terminal, `/record status` still active, `/record stop` succeeds.
7. `/record start` twice in quick succession — second one fails gracefully.
8. Connect AirPods mid-recording — no crash, recording continues.
9. `/record start`, pull the power plug (if laptop), reboot, `/record start` again — stale PID detected, recording succeeds.
10. Zoom call with a colleague for 5 minutes — both sides audible in output.

### Edge Cases to Probe

- macOS 13.0.0 exactly (floor version).
- Intel Mac (x64) — at least one full end-to-end pass before each release.
- External USB mic as default input.
- External audio interface as default output.
- Locale with non-ASCII path (e.g., user's name contains an accent) — `~/.domino/` must handle it.
- Disk full during recording — encoder should log error and exit cleanly; file should be finalized up to the failure point.
- Two Claude Code instances running simultaneously — plugin in each; recording started from one, stop attempted from the other. Should work because control socket is shared.

## Performance Considerations

- Target CPU: <3% on an M1 MacBook during a recording (encoder + two capture threads + Ogg writer).
- Target memory: <30 MB RSS.
- Target disk write: <1 MB/min (Opus at 64 kbps ≈ 480 kB/min).
- Ring buffer size: 2 seconds of audio per channel (192,000 f32 samples = 768 kB). Large enough to tolerate brief encoder stalls, small enough that drops are detected quickly.
- Log all dropped samples; any drop in a released build is a bug to triage.

## Migration Notes

N/A — v1 is greenfield, no existing data to migrate.

Forward-compat note for future phases: keep the on-disk format (stereo Opus, mic-left/system-right) stable. Downstream transcription and diarization will depend on this contract. Any format change (e.g., adding per-speaker channels) must be a new file, not a change in place.

## References

- Project context and all architectural decisions: `so_far.md`
- Plan template source: `starter_pack/for_agents/commands/create_plan_generic.md`
- ScreenCaptureKit audio capture: Apple's WWDC 2022 session 10155 and the `SCStreamConfiguration` docs.
- `cpal` crate: https://docs.rs/cpal
- `audiopus` crate: https://docs.rs/audiopus
- `screencapturekit` crate: https://docs.rs/screencapturekit
- Claude Code plugin docs: (confirm exact URL at implementation time; plugin system is evolving)
