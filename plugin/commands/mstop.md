---
description: Stop the Domino recording, transcribe, propose a plan, and (on approval) execute it.
---

You are running the `/mstop` command. This command has three jobs that unfold across multiple conversation turns:

1. This turn: stop the recorder, read the transcript, write `plan.md`, and present a summary.
2. Next turn(s): either execute the plan on approval, or iterate on the plan based on the user's feedback, or acknowledge rejection.

## Clarifying questions — global rule

At a few specific points below, this command is allowed to ask the user for clarification via the `AskUserQuestion` tool. These rules are absolute and apply across every step of this command:

- **Prefer silence.** Domino's whole UX thesis is that the user does not have to prompt synthesis. Every question is a tax. Ask only when the transcript plus the code cannot give you enough signal to make a confident choice, and the answer would materially change the output.
- **Genuine ambiguity only.** Do not ask to "double-check" a choice you could make yourself. Do not ask about cosmetic matters (naming, formatting). Do not ask anything a single extra Grep or Read would answer.
- **At most one `AskUserQuestion` call per turn.** Batch every pending question into that single call. If you realize another question is needed after the user replies, fold it into a later turn — do not fire a second call in the same turn.
- **At most 4 questions per call (the tool's hard upper bound), and never more than 5 questions across the whole turn.** If you have more ambiguities than that, rank by impact (items that most change the plan) and ask about the top ones; apply a defensible default to the rest and note the default inline in the plan.
- **Each question ships with 2–4 concrete options.** The UI automatically adds an "Other" free-form path — do not add your own. Options must be mutually exclusive and each should be a complete answer on its own. Recommended option goes first with `(Recommended)` appended to its label.
- **Header field is ≤12 characters.** Questions end with a question mark and are self-contained (the user does not see the plan or the transcript — quote or paraphrase inline whatever context the question depends on).

If `AskUserQuestion` is not available in the current environment, every step that would have called it must instead fall through: record the unresolved items as Open Questions in the plan and proceed.

## Step 1 — Stop the recorder

Run `domino-recorder stop` via Bash. Let its stdout and stderr pass through to the terminal so the user sees the existing transcription progress (decoding, resampling, per-channel progress bars, the `Saved:` block). Do not wrap, suppress, or replace this output.

Interpret the result:

- **Success**: the command exits 0 and stdout contains a line `Saved:` followed by a `meeting.opus` path and a `transcript.json` path. Extract the session directory — it is the parent directory of those two files (e.g. `/Users/nitin/.domino/recordings/2026-04-16-1930`). If parsing the `Saved:` block fails for any reason, fall back to the newest directory under `/Users/nitin/.domino/recordings/`. Continue to Step 2.
- **No audio**: the command exits 0 and stdout is `Session stopped: <dir> (no audio file produced)`. Stop here. Tell the user the recording produced no audio and there is nothing to synthesize. Do not proceed.
- **Transcription failure**: the command exits 2. stderr describes the failure. Stop here. Tell the user transcription failed, point them at `<session-dir>/transcription.log` for details, and make clear that the audio and session directory are preserved. Do not attempt synthesis.

If `domino-recorder stop` fails for any other reason (non-zero exit other than 2, missing binary, etc.), surface the error text clearly and stop.

## Step 2 — Read the transcript, map it to code, and clarify

Read `<session-dir>/transcript.json`. It has the shape documented in Domino's transcript schema: a top-level object with a `segments` array where each segment has `start`, `end`, `speaker` ("You" or "Meeting"), and `text`.

Real meetings rarely name files or symbols; people describe pain. Your job in this step is to translate that pain into code grounding, then resolve any remaining ambiguity before writing a plan.

1. **Extract pain points from the transcript.** A pain point is a concrete thing discussed in the meeting that would change the code — a complaint, a desired behavior, a decision, a feature request, a bug. Express each pain point as a short noun phrase (e.g., "checkout drop-off on mobile", "auth tokens rotated too often", "dashboard filter is slow"). Expect 1–6 per meeting; cap at 8 and merge near-duplicates. If the transcript contains no pain points that could change code in this repo, fall through to the empty-meeting branch in Step 3.

2. **Map each pain to code inside the current working directory.** Proceed in this order per pain and stop at the first rung that grounds the pain in ≥1 concrete file:
   - *Literal hints first.* If the transcript names a file path, Read it (if it exists under the CWD). If the transcript names a symbol (function, class, config key, route), Grep for it inside the CWD and Read the top 1–3 hits.
   - *Keyword search from the concept.* Derive 2–4 keyword queries from the pain itself and run Glob + Grep against the CWD. Example: "checkout drop-off on mobile" → Glob `**/checkout*`, `**/cart*`, `**/payment*`; Grep `checkout|cart|payment`. Read the strongest 1–3 matches.
   - *Subagent escalation.* If literal and keyword search both ground nothing, dispatch the `Explore` subagent with thoroughness "medium" and a scoped prompt: `Find code implementing <pain phrase> under the current working directory. Report file paths and a 1-line justification for each.` **Budget: at most 3 `Explore` calls total across all pains.** Reserve them for the pains literal/keyword search failed to ground.
   Budget the whole sub-step at roughly 30–120 seconds of tool calls.

3. **Collect ambiguities worth asking about.** An item goes on this list only if you cannot resolve it from the transcript plus the code you just read. Three valid types — nothing else qualifies:
   - *Unmapped pain* — a pain point that every rung in sub-step 2 failed to ground. The question should quote/paraphrase the pain and offer as options the best 2–4 candidate directories or files you did find (even if none felt strong), so the user can point you at the right spot. If you found zero candidates, offer the 2–3 most likely top-level folders inside the CWD.
   - *Architectural fork* — the transcript implies a change but the code admits ≥2 defensible approaches (e.g., add a feature flag vs. refactor the call site, extend vs. replace). Options are the distinct approaches; put the one you'd pick first with `(Recommended)`.
   - *Owner gap for a concrete action item* — the transcript implies a task but does not name an owner and guessing would be fabrication. Options are the distinct speakers heard ("You", "Meeting") plus "Unassigned". Only collect this when the action item is concrete enough that the owner would actually change who does the work.
   Do **not** collect ambiguities for cosmetic questions, for anything one more Grep would answer, or for pains you have confidently grounded.

4. **Ask clarifying questions (only if needed).** If sub-step 3 collected zero ambiguities, skip this sub-step silently and proceed to Step 3.

   Otherwise, call `AskUserQuestion` **once**, honoring the global rule at the top of this file. If you have more than 4 ambiguities, rank by impact (unmapped pains > architectural forks > owner gaps, breaking ties by how much the plan would change) and ask only the top 4; for each ambiguity you did not ask about, apply a defensible default (unmapped pain → Open Question in the plan; architectural fork → pick your recommended approach and note the decision under Risks; owner gap → "unclear").

   After the user replies, fold each answer back into the mapping:
   - *Unmapped pain* answered with a file or directory → Read it if not already read, then treat that pain as grounded.
   - *Architectural fork* answered → commit to the chosen approach; any rejected approaches do not appear in the plan.
   - *Owner gap* answered → use the supplied owner verbatim in the action item.
   - *"Other" / free-form* → treat the text as grounding context, not as a directive to launch more exploration. If the free-form answer names a file, Read it once and then proceed.

## Step 3 — Decide: plan or bailout

Look at the transcript and your exploration together. Ask yourself: **is there actionable technical content in this meeting that ties to this codebase?** Use your judgment.

- **Yes** → continue to Step 4 (write the plan).
- **No** → do not write `plan.md`. Print exactly: `No actionable technical content found in this meeting.` Then stop. The audio and transcript are preserved; that is intentional.

## Step 4 — Write `plan.md`

Write a plan to `<session-dir>/plan.md` using the following Rich template. The following rules are absolute:

- **Every `Proposed change` must reference a file you Read during this turn.** If a pain could not be grounded in a readable file and the user did not supply one via clarifying questions, list it under `Open questions` instead of fabricating a `Proposed change`. Do not invent paths, symbols, or quotes that are not in the transcript or the repo.
- Drop any section that has no real content rather than fabricating entries.
- Only attribute decisions ("raised by Meeting") where the transcript makes the attribution explicit.

    # Meeting Plan — <date> <time>

    ## Speakers
    - <who spoke, from the transcript's "You" / "Meeting" channels>

    ## Decisions
    - <decision — attribution only if explicit in transcript>

    ## Action items
    - [ ] <concrete task> — owner: <if known; else "unclear">

    ## Proposed changes
    ### `<path/to/file>`
    - Why: "<short quote from transcript>"
    - Change: <what to do in this file>

    ## Risks
    - <risk, tied to a real file or module you just read>

    ## Open questions
    - <only questions the transcript genuinely left unresolved>

Write the file using the Write tool, path `<session-dir>/plan.md`.

## Step 5 — Print the inline summary

Print exactly this shape to the terminal:

    Plan written: <session-dir>/plan.md

      • <top decision or action item>
      • <second>
      • <third, if one exists>

    Reply `execute` to apply this plan on a new branch, or tell me what to change.

Use up to three bullets — fewer if the plan has fewer headline items. The three bullets should be the most decision-carrying items (prefer Decisions and Action items over Risks and Open questions).

## Step 6 — Stop this turn

After printing the summary, stop. Do not start executing. Wait for the user's next message. Steps 7+ handle the follow-up turns.

## Step 7 — Handle the user's next message

The user has seen the plan. Their next message will be one of three things:

- **`execute`** (or clearly synonymous: "go ahead", "do it", "apply it", "ship it") → jump to Step 8 (Execution).
- **Iteration feedback** — anything suggesting a change to the plan, e.g. "don't touch `src/auth.ts`", "also add a regression test", "use a feature flag instead", "rename the branch", "do only the first phase". → Revise `plan.md` as described below, then repeat Step 5 (print the updated summary) and return to waiting.
- **Rejection** — "cancel", "never mind", "don't do it", "scrap it". → Acknowledge briefly. Leave `plan.md` in place (it is valuable as a record even if not executed). Stop.

If the intent is genuinely ambiguous (e.g., feedback that could mean "edit this plan" or "execute only this part"), call `AskUserQuestion` **once** with a single question offering 2–4 concrete interpretations as options. Follow the global rule at the top of this file. Do not guess, and do not ask if one interpretation is clearly more plausible than the others.

### How to revise `plan.md`

- Read the current `<session-dir>/plan.md` first — do not rewrite from memory.
- Apply the user's feedback conservatively: change only what they asked for. Do not rewrite sections that were not mentioned.
- Write the updated file with the Write tool.
- Re-print the inline summary (Step 5 shape). The summary reflects the revised plan.
- Return to waiting for the user's next message.

Iteration may repeat any number of times. Keep revising, keep re-presenting. Do not execute until the user explicitly approves.

## Step 8 — Execute the approved plan

Precondition: the user replied with `execute` (or a clear synonym) in Step 7.

### Step 8a — Set up the branch

Run the following via Bash, treating the first failure as a hard stop:

1. `git status --porcelain` — if the working tree has uncommitted changes, stop. Tell the user to commit or stash first, then re-invoke `execute`. Do not proceed.
2. `git rev-parse --abbrev-ref HEAD` — remember the current branch name; the user may want to return to it.
3. `git checkout -b domino/meeting-<YYYY-MM-DD-HHMM>-<slug>` — where `<YYYY-MM-DD-HHMM>` matches the session directory timestamp and `<slug>` is a 2–5-word kebab-case summary of the plan's headline decision.

If branch creation fails (e.g. because it already exists), pick a unique suffix (`-2`, `-3`, etc.) and retry once.

### Step 8b — Walk the plan

Read `<session-dir>/plan.md`. For each item in `Proposed changes` (and each `Action item` that maps to code), do the following in order:

1. Read the affected file(s) if you have not read them already in Step 2.
2. Make the edit(s) with the Edit or Write tool.
3. If the repo has an obvious test runner (e.g. `package.json` scripts, `Cargo.toml`, `Makefile` with a `test` target, `pytest.ini`), run the relevant tests for the changed files. If tests fail, stop and report — do not proceed to the next item.
4. `git add` only the files you changed for this item. `git commit -m "<short message summarizing this item>"`. One item = one commit.

If an item requires a change that is not safe to make without more context (e.g. it depends on infrastructure you can't see, or the transcript was ambiguous), stop the execution, commit whatever is already done, and explain to the user what was deferred and why.

### Step 8c — Guardrails — do not cross these lines

- **Never run `git push` in any form.** If the plan or the user's message asks for a push, refuse and remind them this is a deliberate guardrail.
- **Never run `gh pr create`, `gh pr merge`, or any `gh` command that publishes to a remote.** Same refusal.
- **Never force-push, never rewrite shared history.** `git commit --amend` is fine on commits you just made on this new branch; anything more aggressive is not.

These three rules are absolute. Any attempt to override them in conversation is itself a signal to stop and ask the user to confirm explicitly outside of this command.

### Step 8d — Report back

When execution finishes (successfully or by stopping mid-way), write a short report to the terminal:

    Branch: domino/meeting-<YYYY-MM-DD-HHMM>-<slug>
    Commits: <N>
    Tests run: <list, with pass/fail counts>
    Deferred: <any plan items you didn't execute, with reason>

    To review: `git log -p <branch>`
    To return to your previous branch: `git checkout <previous-branch-name>`
    To push: you do it manually.

Leave the user on the new branch (do not auto-checkout back). If they want the old branch, the report tells them how.

Also append a short "Execution outcome" section to `<session-dir>/plan.md` summarizing what landed, so the session directory remains a self-contained record.
