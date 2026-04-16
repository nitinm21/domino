# Domino — Project Context & Decisions So Far

A living document capturing the project vision, architectural decisions, tradeoffs considered, and current state. Keep this updated as decisions evolve.

---

## 1. Vision

Turn terminal-based coding assistants (Claude Code, Codex) into proactive engineering collaborators that can **attend meetings** alongside the human engineer and translate what happens there into actionable, codebase-aware work.

Today, the flow looks like:
1. Engineer attends a meeting.
2. Engineer opens Claude Code / Codex.
3. Engineer types a summary of what was decided ("we need a FIFO queue for X").
4. Assistant implements.

The envisioned flow:
1. Engineer hits "start recording" inside their terminal.
2. Engineer attends the meeting normally.
3. Engineer hits "stop recording."
4. Assistant synthesizes the meeting against its understanding of the codebase, proposes a concrete plan + tasks, asks for approval, and executes.

The human only presses a few buttons. The assistant does the rest.

---

## 2. Product Principles (non-negotiable)

- **Terminal-native.** No separate macOS app, no separate Windows app, no browser extension. Everything happens inside the coding assistant as a plugin.
- **Trivial onboarding.** User installs the plugin. That's it. No asking them to paste Zoom links, configure audio routing, install drivers, or open settings panels.
- **Cross-platform.** Must work on macOS and Windows from day one. Linux is out of scope for v1.
- **Capture both sides of the conversation.** System/browser audio (what the meeting says) *and* the user's microphone (what the user says). Both are required — a meeting assistant that only hears one side is useless.

---

## 3. Current Scope (v1)

**In scope:** capture audio locally and store it on disk.

**Out of scope for now (explicitly deferred):**
- Transcription
- Diarization
- LLM synthesis of meeting content
- Proactive task generation
- Plan approval UX
- Integration with the assistant's codebase understanding

The decision to scope down to just capture is deliberate: get the hard, platform-specific, trust-sensitive layer rock-solid first. Everything downstream is pure software on top of a file and can iterate fast once the file exists.

---

## 4. Architecture Decision: Local Capture vs. Meeting-Bot

**Considered:** a meeting-bot approach (Recall.ai, Attendee, etc.) where the user pastes a Zoom/Meet/Teams link and a cloud bot joins the call, returning transcripts via webhook.

**Pros of bot approach:** no local audio stack, no OS permissions, speaker-separated tracks for free, cross-platform by construction.

**Cons (decisive):** requires the user to paste a meeting link every time. That violates the "trivial onboarding, terminal-only" principle. It also doesn't cover hallway conversations, in-person meetings, or ad-hoc calls.

**Decision: local capture.** The helper runs on the user's machine, captures whatever the OS is playing + whatever the mic is hearing, and doesn't care what app the meeting is in (Zoom, Teams, Meet, Discord, phone call via Bluetooth, in-person with a laptop mic, etc.).

---

## 5. Platform Strategy

### macOS
- **Microphone:** `AVAudioEngine` / CoreAudio. Requires TCC **Microphone** permission, granted once per parent process (the terminal app hosting Claude Code).
- **System audio:** `ScreenCaptureKit` with an audio-only `SCStream` (macOS 13+). Requires TCC **Screen Recording** permission.
- **Minimum OS:** macOS 13 (Ventura). Older versions would force users to install BlackHole, which violates the onboarding principle.
- **No driver install. No virtual audio device. No Audio MIDI Setup ritual.**

### Windows
- **Microphone:** WASAPI capture on the default capture endpoint. Win11 has a mic privacy toggle that may need to be enabled once.
- **System audio:** WASAPI loopback on the default render endpoint. Built into Windows since Vista. **No driver, no install, no permission prompt.**
- **Minimum OS:** Windows 10.

### Why not Linux (for v1)
PulseAudio vs. PipeWire fragmentation, distro variance, and smaller user base for the target persona (engineers using Claude Code / Codex on their daily-driver laptops, which are overwhelmingly macOS or Windows). Revisit in v2.

---

## 6. Implementation Language: Rust

**Decision: write the capture helper in Rust.**

**Why Rust:**
- `cpal` crate handles cross-platform audio capture — WASAPI on Windows (including loopback) and CoreAudio on macOS (mic).
- `screencapturekit-rs` provides FFI bindings for macOS system audio capture via ScreenCaptureKit.
- `opus` crate handles encoding.
- Single codebase with `#[cfg(target_os = "…")]` for the ~100 lines of platform-specific glue.
- Produces small, self-contained, dependency-free binaries (~5 MB each) — ideal for bundling inside a plugin.
- Fast startup, low memory, no runtime to ship.

**Alternatives considered:**
- Swift for macOS + C#/C++ for Windows. More code, two build pipelines, two sets of bugs. Rejected.
- Node.js native addons. Heavier runtime, node-gyp pain across platforms. Rejected.
- Bundling `ffmpeg`. Works-ish but fighting filter graphs for WASAPI loopback and ScreenCaptureKit is more work than a 500-line Rust binary, and ffmpeg is 50+ MB.

Estimated total helper size: ~500 lines of Rust.

---

## 7. Output Format: Single Stereo Opus File

**Decision: one file per session, stereo Opus, mic on the left channel, system audio on the right.**

### Options considered

| Option | Pros | Cons |
|---|---|---|
| Two separate files (`mic.opus`, `system.opus`) | Cleanest separation, simplest to reason about | Two files to manage downstream |
| Mono mix of both | Smallest file, simplest | **Information loss** — can't separate user from room; duplicate audio if mic picks up speakers causes transcription artifacts |
| Stereo split (L=mic, R=system) ✅ | One file, zero information loss, trivial to split later with `ffmpeg -map_channel` | Marginally more code than mono |
| Multitrack MKV/MP4 | True separation in one container | Many transcription APIs don't accept multitrack; downstream has to demux anyway |

### Why stereo split wins

- One file to manage — matches the user's mental model ("the recording of my meeting").
- Zero information loss — downstream can always split:
  ```bash
  ffmpeg -i meeting.opus -map_channel 0.0.0 mic.wav      # user only
  ffmpeg -i meeting.opus -map_channel 0.0.1 system.wav   # meeting only
  ```
- Enables echo cancellation later. If the mic picks up the meeting through the user's speakers, having the clean system track lets downstream cancel the echo from the mic track. A mono mix makes this impossible.
- Enables trivial speaker labeling. Left channel = the user. Right channel = everyone else. No voice-fingerprinting guesswork needed.

### Encoding parameters

- **Codec:** Opus (speech-tuned, excellent quality at low bitrates, widely supported).
- **Bitrate:** 64 kbps stereo (~32 kbps/channel, speech quality).
- **Sample rate:** 48 kHz.
- **Container:** `.opus` (Ogg Opus).
- **Expected size:** ~30 MB per hour of meeting.

---

## 8. User-Facing Interface

Slash commands inside the coding assistant:

```
/record start    # begin capturing; returns immediately (non-blocking)
/record stop     # stop capture, flush file
/record status   # is a recording in progress? how long? where?
/record doctor   # check permissions, audio devices, OS version
```

### First-run behavior

**macOS:**
```
> /record start
This is your first recording. Domino needs two macOS permissions:
  • Microphone       (to capture your voice)
  • Screen Recording (to capture meeting audio)

Opening System Settings → Privacy & Security...
After you click Allow for both, re-run: /record start
```

**Windows:**
```
> /record start
Recording started. Session: ~/.claude/recordings/2026-04-15-1423/
```

### Subsequent runs

Silent start on both platforms. No prompts, no friction.

### Stop

```
> /record stop
Saved: ~/.claude/recordings/2026-04-15-1423/meeting.opus (24.1 MB, 47m 12s)
```

---

## 9. Packaging & Distribution

The Rust helper is compiled once per platform target and bundled inside the plugin package:

- `darwin-arm64` (Apple Silicon)
- `darwin-x64`  (Intel Mac)
- `win32-x64`   (most Windows laptops)
- `win32-arm64` (Windows on ARM)

Plugin postinstall selects the correct binary. Pattern follows `esbuild`, `swc`, `better-sqlite3`.

### Code signing (not optional)

- **macOS:** Apple Developer ID ($99/yr) + notarization pipeline. Unsigned binaries trigger Gatekeeper and kill onboarding trust.
- **Windows:** Code-signing certificate. Azure Trusted Signing is the modern cheap path; traditional EV certs are $200–400/yr. Unsigned triggers SmartScreen.

**Budget for signing on day one.** Retrofitting it after users report "is this malware?" is expensive in trust.

---

## 10. Process Lifecycle & Control

`/record start` must return immediately so the terminal remains usable. The recorder runs as a detached background process:

- **macOS:** `launchd` agent OR a plain detached process with a PID file.
- **Windows:** detached child process with a PID file.

Control channel between the plugin and the running recorder:
- PID file at `~/.claude/recordings/current.pid`
- Local Unix socket (macOS) / named pipe (Windows) for `stop`, `status` commands.

The recording must survive the terminal closing mid-meeting — users will close terminals, switch apps, lock the laptop. Only an explicit `/record stop`, a system shutdown, or an OS-initiated device change should end the capture.

---

## 11. Known Gotchas & How We'll Handle Them

### macOS TCC permissions are per-parent-process
If a user runs Claude Code from Terminal.app today and iTerm tomorrow, they re-grant Screen Recording + Microphone. This is Apple's model; we can't change it. `/record doctor` will detect missing permissions and explain exactly what to click.

### Sample-rate changes mid-recording
Bluetooth headsets connecting/disconnecting cause CoreAudio and WASAPI to switch sample rates. Naive capture glitches or crashes. Both APIs expose a format-change callback; we must handle it and resample on the fly.

### Echo when user doesn't wear headphones
Mic picks up the meeting through speakers → mic channel contains both the user's voice and the meeting audio. Stereo split preserves the clean system channel so downstream echo cancellation is possible. Mono mix would have made this unrecoverable.

### Exclusive-mode audio apps
Rarely, apps request exclusive audio device access. WASAPI loopback taps the system mix (unaffected), and ScreenCaptureKit captures the system mix (unaffected). Should be fine in practice.

### Legal / consent
All-party-consent jurisdictions exist (California, Washington, etc.). First-run UX includes a one-time disclaimer; acceptance is logged locally. Not our legal liability to enforce, but we make it obvious.

### File size
Raw 48 kHz stereo WAV = ~10 MB/min/stream. Encoding to Opus inline (not post-hoc) keeps a one-hour meeting at ~30 MB instead of ~1 GB. Do it in the recorder, not as a cleanup step.

---

## 12. Testing Matrix

Before any release, smoke-test on:

| OS | Arch | Notes |
|---|---|---|
| macOS 13 | arm64 | minimum supported |
| macOS 14 | arm64 | |
| macOS 15 | arm64 | latest |
| macOS 14 | x64   | Intel Mac regression |
| Windows 10 | x64 | minimum supported |
| Windows 11 | x64 | majority platform |
| Windows 11 | arm64 | Surface / Copilot+ PCs |

Each smoke test: start, speak, play a YouTube video for 30s, stop, verify both channels have audio in the output file.

---

## 13. Build Order (suggested sequence)

1. **Day 1 — Windows capture.** Easier platform; validates the core approach. WASAPI loopback + mic → stereo Opus file.
2. **Day 2 — macOS mic.** AVAudioEngine capture. TCC prompt handling.
3. **Day 3 — macOS system audio.** ScreenCaptureKit integration. This is the hardest single piece.
4. **Day 4 — Signing & notarization pipelines.** Both platforms.
5. **Day 5 — Plugin shell & slash commands.** Wrapper around the now-solid recorder binary.
6. **Week 2 — Polish:** `doctor` command, detached lifecycle, socket control channel, sample-rate change handling.

---

## 14. Open Questions

- Exact plugin API surface for Claude Code vs. Codex — do they share a plugin manifest format or do we ship two thin wrappers around the same binary?
- Where should recordings live by default? `~/.claude/recordings/` works for Claude Code but may not be the right default for Codex users. Possibly `~/.domino/recordings/` with an env override.
- Automatic cleanup policy — delete recordings after N days? After downstream processing succeeds? User-configurable?
- Should `/record start` default to both channels, or expose flags for mic-only / system-only? Likely both by default; flags for power users.

---

## 15. Decisions Log (chronological)

| Date | Decision | Why |
|---|---|---|
| 2026-04-15 | Terminal-native plugin, not a separate Mac/Windows app | Onboarding simplicity is the core product principle |
| 2026-04-15 | Local capture, not meeting-bot API | Bot approach requires pasting meeting links, violates zero-config UX |
| 2026-04-15 | Rust for the capture helper | Cross-platform audio via `cpal` + `screencapturekit-rs`, small binaries, no runtime |
| 2026-04-15 | macOS 13+ minimum (no BlackHole) | ScreenCaptureKit gives driver-free system audio; BlackHole would reintroduce install friction |
| 2026-04-15 | Single stereo Opus file, mic-left / system-right | One file for downstream simplicity, zero information loss vs. two files, enables echo cancellation & speaker labeling later |
| 2026-04-15 | v1 scope limited to capture + local storage | De-risk the platform-specific layer before building transcription/synthesis on top |
