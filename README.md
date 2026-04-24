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

More information: https://domino-meet.vercel.app/

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


