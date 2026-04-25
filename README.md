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

You finish a meeting where ten things changed at once: the API shape is different, one edge case needs a fix, a migration has to happen before release, and somebody needs to update the docs so the rest of the team does not build against stale assumptions. Everyone leaves aligned, but the real work is still trapped inside the conversation until someone sits down and translates it into a plan your agent can execute.

Domino does that for you. It records the meeting and transcribes it locally. With its understanding of the codebase, it writes a grounded implementation plan (grounded against your codebase) you can execute. Instead of relying on memory and scattered notes, you leave the meeting with work that is already structured and ready to execute.

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


