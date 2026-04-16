# Claude Folder Overview

This folder is a small Claude workspace with a local `settings.json` and a `.claude` directory that stores skills, commands, and agents. Below is a quick guide to what is inside.

## Skills

- `spec-driven-dev`: Creates a full `spec.md` before any build work and pushes for clear, testable requirements.
- `stop-slop`: Cleans up prose by removing common AI writing tells and filler.
- `ui-skills`: Defines strict UI rules for accessibility, layout, animation, and design consistency.

## Commands

- `create_plan_generic`: Guides a deep planning workflow that uses research agents and produces a phased implementation plan.
- `debug`: Starts a structured debugging session focused on logs, database state, and git history without editing files.
- `implement_plan`: Executes an approved plan step by step, runs checks, and tracks progress in the plan file.
- `research_codebase_generic`: Orchestrates parallel research and writes a formal research document with citations.
- `spec`: Creates a comprehensive `spec.md` for new projects using a spec driven development flow.

## Agents

- `codebase-analyzer`: Explains how code works by tracing data flow and behavior with file and line references.
- `codebase-locator`: Finds where relevant files live and groups them by purpose without analyzing content.
- `codebase-pattern-finder`: Locates similar implementations and surfaces concrete code examples.
- `thoughts-analyzer`: Extracts high value insights from thoughts documents and filters out noise.
- `thoughts-locator`: Searches the thoughts directory and organizes documents by type.
- `web-search-researcher`: Runs web research with sources and quotes when the answer is not in the codebase.
