# What is Domino?

Domino is a meeting-aware thinking partner. You record a working conversation — a standup, a customer interview, a planning session — and Domino turns it into structured, grounded artifacts spread across the discrete projects the conversation actually touched.

## The problem it solves

After most working conversations, someone has to sit down and translate what was said into edits across many places: tickets in one project, spec changes in another, code in a third, follow-ups for a fourth. That translation is tedious, lossy, and often skipped. Domino does the fan-out automatically so the human can spend their time deciding, not transcribing.

## How it works

1. Record the meeting (`/mstart` … `/mstop`).
2. Domino transcribes and diarizes the audio locally.
3. It scans the directory you launched from, identifies the projects underneath it, and **routes** each segment of the transcript to the project it concerns.
4. For each project, it **synthesizes** a grounded plan — pain points mapped to real files, decisions, action items, open questions — written into the session directory.
5. Optionally, it can **execute** the plan: spawning a scoped subprocess per project, creating a branch, and committing the changes one at a time.

## The thesis

Routing and synthesis are the heart of the product. Execution is a convenience, not the point. Domino is not trying to replace the human — it is trying to be the sounding board that has already done the boring fan-out work, so the human can show up to a structured, grounded starting place per project and decide what to do next.

## Where it could go

The same routing-and-synthesis pattern applies anywhere conversations drive work across discrete buckets: customer research, sales calls, legal matters, editorial planning, portfolio reviews. The substrate today is folders and git; tomorrow it could be Notion pages, Linear projects, or Drive folders.
