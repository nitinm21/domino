"use client";

import { motion, type Variants } from "framer-motion";
import { CodeBlock } from "./CodeBlock";

const container: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.08, delayChildren: 0.05 } },
};

const item: Variants = {
  hidden: { opacity: 0, y: 18 },
  show: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.7, ease: [0.21, 0.47, 0.32, 0.98] },
  },
};

function VideoIcon({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
      className={className}
    >
      <rect x="2" y="6" width="14" height="12" rx="2.2" />
      <path d="M22 8v8a.6.6 0 0 1-.97.47L16 13v-2l5.03-3.47A.6.6 0 0 1 22 8Z" />
    </svg>
  );
}

function MergeIcon({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className={className}
    >
      <circle cx="6" cy="4" r="2.3" />
      <circle cx="6" cy="20" r="2.3" />
      <circle cx="18" cy="20" r="2.3" />
      <path d="M6 6.3v11.4" />
      <path d="M6 12q12 0 12 5.7" />
    </svg>
  );
}

function IconChip({
  children,
  tone,
}: {
  children: React.ReactNode;
  tone: "accent" | "merge";
}) {
  const toneClasses =
    tone === "merge"
      ? "bg-merge/[0.1] text-merge ring-merge/25"
      : "bg-accent/[0.09] text-accent ring-accent/25";
  return (
    <span
      aria-hidden="true"
      className={`mx-[0.08em] inline-flex h-[0.92em] w-[0.92em] translate-y-[-0.03em] items-center justify-center rounded-[0.2em] ring-1 ring-inset align-middle ${toneClasses}`}
    >
      {children}
    </span>
  );
}

export function Hero() {
  return (
    <section className="relative pb-2 pt-6 md:pt-10">
      <div className="hero-grid" aria-hidden />
      <motion.div variants={container} initial="hidden" animate="show" className="relative">
        <motion.h1
          variants={item}
          className="mb-5 text-[clamp(40px,6vw,64px)] font-semibold leading-[1.08] tracking-[-0.035em]"
        >
          <span className="whitespace-nowrap">
            From{" "}
            <IconChip tone="accent">
              <VideoIcon className="h-[0.58em] w-[0.58em]" />
            </IconChip>{" "}
            meeting,
          </span>
          <br />
          <span className="whitespace-nowrap">
            to{" "}
            <IconChip tone="merge">
              <MergeIcon className="h-[0.66em] w-[0.66em]" />
            </IconChip>{" "}
            merge.
          </span>
        </motion.h1>
        <motion.p
          variants={item}
          className="mb-9 max-w-[620px] text-[18px] leading-relaxed text-ink-muted"
        >
          Domino records meetings inside Claude Code, transcribes it, and writes a grounded
          implementation plan you can execute.
        </motion.p>
        <motion.div variants={item} className="mb-4">
          <CodeBlock
            code="curl -fsSL https://raw.githubusercontent.com/nitinm21/domino/main/install.sh | sh"
            copyLabel="Copy install command"
            className="max-w-[640px]"
            preClassName="whitespace-nowrap"
            prefix="$ "
          />
        </motion.div>
        <motion.p variants={item} className="text-sm text-ink-muted">
          macOS 14+ on Apple Silicon. Requires{" "}
          <a
            href="https://claude.com/claude-code"
            className="text-accent underline-offset-[3px] transition-colors hover:text-accent-hover hover:underline"
          >
            Claude Code
          </a>{" "}
          <span>(Codex support coming soon!)</span>
          .
        </motion.p>
      </motion.div>
    </section>
  );
}
