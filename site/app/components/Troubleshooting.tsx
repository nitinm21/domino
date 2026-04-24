"use client";

import { AnimatePresence, motion } from "framer-motion";
import { useState } from "react";
import type { ReactNode } from "react";
import { CodeBlock } from "./CodeBlock";
import { SectionHeader } from "./SectionHeader";

type Item = { id: string; head: ReactNode; body: ReactNode };

function items(): Item[] {
  return [
    {
      id: "not-found",
      head: (
        <>
          Why does Claude Code/Codex say <code>domino-recorder: command not found</code>?
        </>
      ),
      body: (
        <>
          <p className="mb-3">
            Run the curl installer first. The plugin shells out to{" "}
            <code>domino-recorder</code> on your PATH.
          </p>
          <h4 className="mb-2 font-semibold text-ink">Claude Code</h4>
          <CodeBlock
            code="curl -fsSL https://raw.githubusercontent.com/nitinm21/domino/main/install.sh | sh"
            copyLabel="Copy Claude Code install command"
          />
          <h4 className="mb-2 mt-4 font-semibold text-ink">Codex</h4>
          <CodeBlock
            code="curl -fsSL https://raw.githubusercontent.com/nitinm21/domino-codex/main/install.sh | sh"
            copyLabel="Copy Codex install command"
          />
        </>
      ),
    },
    {
      id: "xcrun",
      head: (
        <>
          What should I do if I see <code>xcrun: error: invalid active developer path</code>{" "}
          or missing Swift libraries?
        </>
      ),
      body: (
        <>
          <p className="mb-2.5">Install the Xcode Command Line Tools:</p>
          <CodeBlock code="xcode-select --install" copyLabel="Copy Xcode install command" />
        </>
      ),
    },
    {
      id: "intel",
      head: <>What if I&apos;m on an Intel Mac?</>,
      body: (
        <p>
          Domino currently supports Apple Silicon Macs only, starting with M1. Intel Macs
          are not supported at this time.
        </p>
      ),
    },
    {
      id: "gatekeeper",
      head: <>What if macOS Gatekeeper blocks the binary?</>,
      body: (
        <>
          <p className="mb-2.5">
            The installer strips the quarantine attribute automatically. If you downloaded
            the binary manually:
          </p>
          <CodeBlock
            code="xattr -d com.apple.quarantine /usr/local/bin/domino-recorder"
            copyLabel="Copy Gatekeeper workaround command"
          />
        </>
      ),
    },
    {
      id: "perms",
      head: <>What if the permissions prompt didn&apos;t appear?</>,
      body: (
        <p>
          Open{" "}
          <strong className="font-semibold text-ink">
            System Settings → Privacy &amp; Security → Microphone
          </strong>{" "}
          and <strong className="font-semibold text-ink">Screen Recording</strong>, and add
          your terminal to the allowed list. Restart the app after granting.
        </p>
      ),
    },
    {
      id: "usage",
      head: <>Does transcription count against Claude Code/Codex usage limits?</>,
      body: (
        <p>
          No. Transcription runs on-device using Whisper. Only plan generation and
          execution count against Claude Code/Codex usage limits.
        </p>
      ),
    },
  ];
}

export function Troubleshooting() {
  const [open, setOpen] = useState<string | null>(null);
  const list = items();

  return (
    <section id="troubleshooting" className="mt-24">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, margin: "0px 0px -80px 0px" }}
        transition={{ duration: 0.65, ease: [0.21, 0.47, 0.32, 0.98] }}
      >
        <SectionHeader title="FAQs" />
        <div>
          {list.map((it) => {
            const isOpen = open === it.id;
            return (
              <div key={it.id} className="border-b border-rule">
                <button
                  type="button"
                  onClick={() => setOpen(isOpen ? null : it.id)}
                  aria-expanded={isOpen}
                  className="flex w-full items-center justify-between gap-4 py-3.5 text-left font-medium text-ink transition-colors hover:text-accent"
                >
                  <span>{it.head}</span>
                  <motion.span
                    animate={{ rotate: isOpen ? 45 : 0 }}
                    transition={{ duration: 0.2, ease: "easeOut" }}
                    className="flex h-5 w-5 items-center justify-center font-mono text-lg leading-none text-ink-faint"
                    aria-hidden
                  >
                    +
                  </motion.span>
                </button>
                <AnimatePresence initial={false}>
                  {isOpen && (
                    <motion.div
                      key="content"
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{
                        height: { duration: 0.3, ease: [0.21, 0.47, 0.32, 0.98] },
                        opacity: { duration: 0.2 },
                      }}
                      className="overflow-hidden"
                    >
                      <div className="pb-4 pt-1 leading-relaxed text-ink-muted">
                        {it.body}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            );
          })}
        </div>
      </motion.div>
    </section>
  );
}
