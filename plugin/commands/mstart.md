---
description: Start a Domino recording session (mic + system audio, macOS).
---

## Preflight

1. Run `command -v domino-recorder` via Bash. If it exits non-zero, stop and print:

   ```
   domino-recorder is not installed. Install with:

       curl -fsSL https://raw.githubusercontent.com/nitinm21/domino/main/install.sh | sh

   Then re-run /mstart.
   ```

   Do not proceed to the recorder step.

2. Check for the first-run acknowledgment: `test -f ~/.domino/acknowledged` via Bash. If the file does NOT exist, this is a first run on this machine. Print the banner below verbatim, then use AskUserQuestion (header: "First run", options: "Continue" / "Cancel") to confirm. Do not start the recorder until the user picks Continue.

   ```
   === Domino first-run setup ===

   Two things will happen the first time you record:

   1. macOS will prompt for Microphone access and Screen & System Audio
      Recording access. Grant both — they are separate permissions and
      both are required for the meeting capture to work.

   2. After you grant Screen & System Audio Recording, macOS will likely
      tell you to "quit and reopen" Claude Code so it can pick up the new
      permission. This is a macOS rule — permissions are cached per
      process. Do it: quit Claude Code, relaunch it, and then run
      /mstart again.

   You will only see this setup once. Continuing creates the marker file
   ~/.domino/acknowledged.
   ```

   On "Continue", run `mkdir -p ~/.domino && touch ~/.domino/acknowledged` via Bash, then proceed to the recorder step.
   On "Cancel", stop. Do not start the recorder. Do not create the marker file.

## Start the recorder

Run `domino-recorder start` via Bash. Print its stdout verbatim (it's session JSON: pid, session_dir, started_at). If the command exits non-zero, surface the error text clearly and do nothing else — the recorder's error message already tells the user what to do.

Do not read files. Do not explore the repo. Do not offer further commentary — this command exists only to start the recorder and get out of the way.
