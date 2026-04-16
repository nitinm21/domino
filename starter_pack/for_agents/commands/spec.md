---
description: Create a comprehensive spec.md file before building any project. Follows the spec-driven development workflow with product definition + phased implementation tasks.
allowed-tools: Read, Write, Glob, Grep, WebFetch, Task
---

# Spec-Driven Development

You are creating a comprehensive specification for a new project. Follow this exact workflow:

## Step 1: Gather Context

Before writing anything:
1. Read any data files, existing code, or references the user has mentioned
2. Ask clarifying questions if the requirements are vague
3. Identify the tech stack (or recommend: Next.js + Tailwind + Framer Motion for web apps)
4. Understand scope: Is this a demo, MVP, or production system?

## Step 2: Create spec.md

Create a `spec.md` file at the project root with TWO parts:

### Part 1: Product Definition (Eliminate ALL Ambiguity)

Include these sections:
- **1.1 What We Are Building**: 2-3 paragraphs on the core product
- **1.2 The Core Experience**: Emotional/functional goals, how users should feel
- **1.3 Visual Design Language**: Exact hex colors, typography, spacing, animation principles
- **1.4 The Layout**: ASCII diagram + section specifications
- **1.5 Core Functionality**: Every feature described with user actions, system responses, edge cases
- **1.6 Data Flow**: How data moves through the system (diagram if helpful)
- **1.7 Desired User Reactions**: Table of moments → expected reactions
- **1.8 What This Is NOT**: Explicit out-of-scope list

### Part 2: Technical Implementation (Phased Tasks)

Structure as exactly THREE phases:

**Phase 1: Foundation** (~3 tasks)
- Project setup, data layer, base layout

**Phase 2: Core Components** (~3-4 tasks)
- Main interactive components, one per task

**Phase 3: Polish & Integration** (~3 tasks)
- Animations, connections, final refinements

Each task must have:
- **What**: One sentence description
- **Acceptance Criteria**: Bullet point checklist
- **Commit message**: Suggested commit text

## Principles

1. **Zero Ambiguity**: Reader should visualize the exact same product
2. **Atomic Tasks**: Each task completable in one session
3. **Incremental Value**: Each task produces something visible
4. **Commit After Each**: Every task ends with a commit

## After Creating

Tell the user: "I've created spec.md. Ready to start with Task 1.1: [Task Name]?"

Then execute ONE task at a time, committing after each.
