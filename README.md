# Domino

Domino records meetings inside your coding agent, transcribes it, and writes a grounded implementation plan you can execute.
This contains installation instructions for Claude Code. For Codex CLI installation, go here: https://github.com/nitinm21/domino-codex/

## Install

1. Install the recorder binary (macOS, Apple Silicon):

   ```bash
   curl -fsSL https://raw.githubusercontent.com/nitinm21/domino/main/install.sh | sh
   ```

2. Inside Claude Code, add the plugin marketplace and install:

   ```
   /plugin marketplace add nitinm21/domino
   /plugin install domino@domino
   ```

3. Record a meeting:

   ```
   /mstart
   … hold the meeting …
   /mstop
   ```

## What it does

After most working conversations, someone has to sit down and translate what was said into edits across many places: tickets in one project, spec changes in another, code in a third, follow-ups for a fourth. That translation is tedious, lossy, and often skipped. Domino does the fan-out automatically so the human can spend their time deciding, not transcribing.

The flow:

1. You record the meeting with `/mstart` and `/mstop`.
2. Domino transcribes and diarizes the audio locally.
3. It scans the directory you launched Claude Code from, identifies the projects underneath it, and routes each segment of the transcript to the project it concerns.
4. For each project, it synthesizes a grounded plan — pain points mapped to real files, decisions, action items, open questions — written into the session directory.
5. Optionally, it can execute the plan: spawning a scoped subprocess per project, creating a branch, and committing the changes one at a time.

Routing and synthesis are the heart of the product. Execution is a convenience, not the point. Domino is not trying to replace the human — it is trying to be the sounding board that has already done the boring fan-out work, so the human can show up to a structured, grounded starting place per project and decide what to do next.

## Requirements

- macOS 14+ on Apple Silicon.
- Xcode Command Line Tools (`xcode-select --install`). The recorder links against Swift's concurrency runtime via ScreenCaptureKit.
- A Claude Code subscription.
- Roughly 500 MB of disk for the first-run Whisper model download.

## Privacy

- **Audio stays on the device.** The Opus file lives under `~/.domino/recordings/` and is never uploaded anywhere by this plugin.
- **Transcription is local.** Whisper runs on your machine via the bundled model; no audio leaves the device during transcription.
- **Synthesis uses your Claude Code session.** During `/mstop`, the transcript text (and any repo files Claude reads to ground the plan) is sent to Anthropic via your existing Claude Code subscription. Treat the transcript the same way you treat anything you paste into Claude Code.
- **Execution is local.** Branch creation, edits, tests, and commits all happen in your working copy. The plugin never runs `git push` and never opens a PR.

## Commands

- `/mstart` — start recording (mic + system audio).
- `/mstat` — show the active session, or `{}` if idle.
- `/mstop` — stop, transcribe locally, synthesize a plan grounded in the current repo, optionally execute on a branch.

## Where recordings live

Every meeting gets its own directory under `~/.domino/recordings/<YYYY-MM-DD-HHMM>/` containing:

- `meeting.opus` — the stereo Opus recording (left = mic, right = system audio)
- `transcript.json` — the structured transcript
- `recorder.log` — daemon log
- `transcription.log` — transcription phase log
- `plan.md` — written after `/mstop` if synthesis produced actionable content

## Troubleshooting

- **`domino-recorder: command not found`** inside Claude Code. Run the curl installer first — the plugin shells out to `domino-recorder` on your PATH.
- **`xcrun: error: invalid active developer path`** or missing Swift libraries. Run `xcode-select --install`.
- **Intel Mac.** v0.1.0 does not ship a prebuilt x86_64 binary. Build from source: `cargo build --release --manifest-path recorder/Cargo.toml`, then copy `recorder/target/release/domino-recorder` to `/usr/local/bin`.
- **macOS Gatekeeper blocks the binary.** The installer strips the quarantine attribute automatically. If you downloaded the binary manually, run `xattr -d com.apple.quarantine /usr/local/bin/domino-recorder`.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for plugin-development details (local plugin loading, reload workflow, plugin conventions).

## License

MIT — see [LICENSE](./LICENSE).
