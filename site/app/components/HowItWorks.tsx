"use client";

import { motion, type Variants } from "framer-motion";
import type { ReactNode } from "react";
import { SectionHeader } from "./SectionHeader";

const container: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.08 } },
};

const item: Variants = {
  hidden: { opacity: 0, y: 18 },
  show: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.6, ease: [0.21, 0.47, 0.32, 0.98] },
  },
};

const TRUST: Array<{ head: string; body: ReactNode }> = [
  {
    head: "On-device.",
    body: "Transcription happens locally with Whisper; audio never leaves your machine.",
  },
  {
    head: "Grounded.",
    body: "The plan cites your actual code: file-level changes, rationale, risks, open questions.",
  },
  {
    head: "Local only.",
    body: "If you choose to execute, Domino creates a branch and commits locally. Never pushes. Never opens a PR.",
  },
];

export function HowItWorks() {
  return (
    <section id="how" className="mt-24">
      <motion.div
        variants={container}
        initial="hidden"
        whileInView="show"
        viewport={{ once: true, margin: "0px 0px -80px 0px" }}
      >
        <motion.div variants={item}>
          <SectionHeader title="How it works" />
        </motion.div>

        <motion.p
          variants={item}
          className="mb-8 max-w-prose text-[17px] leading-relaxed text-ink-muted"
        >
          You record. Domino does the rest.
        </motion.p>

        <motion.div variants={item} className="terminal-window mb-10">
          <div className="terminal-titlebar">
            <span className="terminal-dot bg-[#ff5f57]" aria-hidden />
            <span className="terminal-dot bg-[#febc2e]" aria-hidden />
            <span className="terminal-dot bg-[#28c840]" aria-hidden />
          </div>
          <pre className="codeblock">
            <code>
              <span className="codeblock-prefix">$ </span>/mstart{"\n"}
              <span className="codeblock-prefix">    </span>recording meeting…{"\n"}
              {"\n"}
              <span className="codeblock-prefix">$ </span>/mstop{"\n"}
              <span className="codeblock-prefix">    ├─ </span>whisper: transcribing on-device{"\n"}
              <span className="codeblock-prefix">    ├─ </span>scanning repo: grounding plan{"\n"}
              <span className="codeblock-prefix">    ├─ </span>plan ready{"\n"}
              <span className="codeblock-prefix">    └─ </span>execute plan or chat about it{"\n"}
            </code>
          </pre>
        </motion.div>

        <motion.h3
          variants={item}
          className="mb-4 text-[17px] font-semibold tracking-[-0.01em] text-ink"
        >
          Under the hood:
        </motion.h3>
        <ul className="m-0 list-none p-0">
          {TRUST.map((t) => (
            <motion.li key={t.head} variants={item} className="bullet">
              <strong className="font-semibold text-ink">{t.head}</strong> {t.body}
            </motion.li>
          ))}
        </ul>
      </motion.div>
    </section>
  );
}
