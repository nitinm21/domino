"use client";

import { AnimatePresence, motion } from "framer-motion";
import { CodeBlock } from "./CodeBlock";
import { Reveal } from "./Reveal";
import { SectionHeader } from "./SectionHeader";
import { ToolTabs } from "./ToolTabs";
import { useTool } from "./ToolContext";

const ROWS = (prefix: "/" | "$"): Array<[string, React.ReactNode]> => [
  [`${prefix}mstart`, "Start recording. Captures mic and system audio."],
  [
    `${prefix}mstat`,
    <>
      Show the active session, or <code>{"{}"}</code> if idle.
    </>,
  ],
  [
    `${prefix}mstop`,
    "Stop, transcribe locally, synthesize plans, optionally execute on a branch.",
  ],
];

export function Commands() {
  const { tool } = useTool();
  const prefix = tool === "claude-code" ? "/" : "$";
  const rows = ROWS(prefix);

  return (
    <section id="commands" className="mt-24">
      <Reveal>
        <SectionHeader title="Commands" />
        <div className="mb-3">
          <ToolTabs layoutId="tool-tabs-commands" />
        </div>
        <AnimatePresence mode="wait" initial={false}>
          <motion.div
            key={tool}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -6 }}
            transition={{ duration: 0.22, ease: [0.21, 0.47, 0.32, 0.98] }}
            className="grid grid-cols-[max-content_1fr] items-baseline gap-x-6 gap-y-2.5"
          >
            {rows.map(([cmd, desc]) => (
              <div key={cmd} className="contents">
                <div>
                  <code className="whitespace-nowrap">{cmd}</code>
                </div>
                <div className="text-ink-muted">{desc}</div>
              </div>
            ))}
          </motion.div>
        </AnimatePresence>
        <h3 className="mb-2 mt-9 text-[17px] font-semibold tracking-[-0.01em]">
          Where recordings live
        </h3>
        <p className="mb-3.5 leading-relaxed text-ink-muted">
          Every meeting gets its own directory under{" "}
          <code>~/.domino/recordings/&lt;YYYY-MM-DD-HHMM&gt;/</code> containing:
        </p>
        <CodeBlock
          code={`meeting.opus         — stereo Opus recording (L = mic, R = system)
transcript.json      — structured transcript
recorder.log         — daemon log
transcription.log    — transcription phase log
plan.md              — written after ${prefix}mstop if synthesis produced actionable content`}
          copyLabel="Copy recording directory contents"
        />
      </Reveal>
    </section>
  );
}
