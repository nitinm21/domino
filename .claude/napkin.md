# Napkin Runbook

## Curation Rules
- Re-prioritize on every read.
- Keep recurring, high-value notes only.
- Max 10 items per category.
- Each item includes date + "Do instead".

## Execution & Validation (Highest Priority)
1. **[2026-04-16] Verify Domino from the Rust recorder binary first**
   Do instead: use `recorder/target/release/domino-recorder` for manual checks before assuming any plugin workflow exists.
2. **[2026-04-16] Treat phase coverage as partial until the plan says otherwise**
   Do instead: check the relevant phase section in `thoughts/shared/plans/2026-04-15-domino-v1-macos-audio-capture.md` and then confirm the matching code paths in `recorder/src/`.
3. **[2026-04-16] Concurrency automation is not green yet**
   Do instead: if `cargo test` fails in `tests/concurrent_start.rs`, rely on direct manual `start`/`status`/`stop` verification for lifecycle behavior and call out the gap explicitly.
4. **[2026-04-16] Phase 3 system-audio verification depends on logs, not `status`**
   Do instead: inspect `<session>/recorder.log` for `starting system audio capture via ScreenCaptureKit` or the mic-only fallback warning because `status` still only prints PID/session metadata.

## Shell & Command Reliability
1. **[2026-04-16] `starter_pack` must stay as plain files in the top-level repo**
   Do instead: keep `starter_pack/.git` out of the workspace before staging so Git tracks the folder contents instead of an embedded repo link.
2. **[2026-04-16] Build commands are anchored on the recorder manifest**
   Do instead: run `cargo build --release --manifest-path recorder/Cargo.toml` or `cargo test --manifest-path recorder/Cargo.toml` from repo root.
3. **[2026-04-16] `screencapturekit` rebuilds can fail on local Swift toolchain mismatches**
   Do instead: if a fresh `cargo build` fails in the custom build step, use the existing compiled binary for manual recorder verification and treat the Swift environment as a separate toolchain problem.

## Domain Behavior Guardrails
1. **[2026-04-16] `doctor` is still a stub**
   Do instead: do not route users through `domino-recorder doctor` for permissions until Phase 4 lands; use direct macOS permission steps instead.
2. **[2026-04-16] Current manual verification is terminal-driven, not browser-driven**
   Do instead: treat the browser as an optional sound source or meeting simulator; the authoritative checks are the saved Opus file, `status`, `ps`, and `ffprobe`.
3. **[2026-04-16] Keep transcription downstream of the saved session artifact**
   Do instead: treat `~/.domino/recordings/<session>/meeting.opus` as the stable handoff and write transcript outputs beside it rather than adding model work into the live capture loop.
