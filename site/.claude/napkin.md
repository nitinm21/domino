# Napkin Runbook

## Curation Rules
- Re-prioritize on every read.
- Keep recurring, high-value notes only.
- Max 10 items per category.
- Each item includes date + "Do instead".

## Execution & Validation (Highest Priority)
1. **[2026-04-20] Validate marketing-site changes with a production build**
   Do instead: run `npm run build` from `/Users/nitin/domino/site` after changing app-router components or shared CSS.
2. **[2026-04-21] Dev and build artifacts must stay isolated**
   Do instead: keep `next dev` writing to `.next-dev` and `next build` writing to `.next` so validation builds do not break a running dev server.
3. **[2026-04-21] Vercel deploys must run from the `site/` app root**
   Do instead: run `vercel deploy --prod --yes` from `/Users/nitin/domino/site`; Vercel links that folder as the standalone `site` project and writes local metadata into `.vercel/`.
4. **[2026-04-21] Git-connected Vercel project does not infer the monorepo subdirectory**
   Do instead: after `vercel git connect` for `nitinm21/domino`, patch the cloud project with `rootDirectory=site` before relying on GitHub-triggered builds.

## Shell & Git
1. **[2026-04-21] Parent repo git status does not reflect incremental `site/` edits**
   Do instead: treat `/Users/nitin/domino/site` as a locally edited app directory, verify changes by reading files directly and running `npm run build` instead of relying on `git diff` from `/Users/nitin/domino`.
2. **[2026-04-21] Ignored directories can still stay on GitHub if they were tracked earlier**
   Do instead: when a path like `/thoughts` is already in `.gitignore` but still appears in GitHub, remove it from the index with `git rm -r --cached <path>` and commit the deletion without deleting the local files.

## UI System
1. **[2026-04-20] Section navigation depends on stable section IDs**
   Do instead: keep `id` values in section wrappers aligned with `Sidebar.tsx` nav items when restyling or reordering sections.
2. **[2026-04-20] Section headings now use a shared single-heading treatment**
   Do instead: reuse `app/components/SectionHeader.tsx` and the global `.section-*` styles instead of reintroducing eyebrow + oversized headline stacks.
3. **[2026-04-20] Theme colors flow through CSS variables**
   Do instead: update the `--*-rgb` tokens in `app/globals.css` and keep `tailwind.config.ts` bound to those variables instead of hardcoding hex values in components.

## User Directives
1. **[2026-04-20] Keep the landing page typography restrained**
   Do instead: prefer a calm, premium hierarchy with headings only modestly larger than the 17px-18px body copy, closer to Agentation/Linear than loud marketing display styles.
