import { Reveal } from "./Reveal";
import { SectionHeader } from "./SectionHeader";
import type { ReactNode } from "react";

const ITEMS: Array<{ head: string; body: ReactNode }> = [
  {
    head: "Audio stays on device.",
    body: (
      <>
        The Opus file lives under <code>~/.domino/recordings/</code> and is never uploaded by this
        plugin.
      </>
    ),
  },
  {
    head: "Transcription is local.",
    body: (
      <>
        Whisper runs on your machine via the bundled model. No audio leaves the device during
        transcription.
      </>
    ),
  },
  {
    head: "Synthesis uses your Claude Code session.",
    body: (
      <>
        During <code>/mstop</code>, the transcript text and any repo files Claude reads to ground
        the plan are sent to Anthropic via your existing Claude Code subscription. Treat the
        transcript the same way you treat anything you paste into Claude Code.
      </>
    ),
  },
  {
    head: "Execution is local.",
    body: (
      <>
        Branch creation, edits, tests, and commits all happen in your working copy. The plugin
        never runs <code>git push</code> and never opens a PR.
      </>
    ),
  },
];

export function Privacy() {
  return (
    <section id="privacy" className="mt-24">
      <Reveal>
        <SectionHeader title="Privacy" />
        <ul className="m-0 list-none p-0">
          {ITEMS.map((it) => (
            <li key={it.head} className="bullet">
              <strong className="font-semibold text-ink">{it.head}</strong> {it.body}
            </li>
          ))}
        </ul>
      </Reveal>
    </section>
  );
}
