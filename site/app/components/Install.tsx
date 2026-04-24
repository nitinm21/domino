"use client";

import { AnimatePresence, motion } from "framer-motion";
import { Reveal } from "./Reveal";
import { CodeBlock } from "./CodeBlock";
import { SectionHeader } from "./SectionHeader";
import { ToolTabs } from "./ToolTabs";
import { useTool } from "./ToolContext";

function ClaudeCodeSteps() {
  return (
    <ol className="list-decimal space-y-6 pl-5 marker:font-mono marker:text-ink-faint">
      <li className="leading-relaxed">
        <p className="mb-2.5">
          <strong className="font-semibold">Install the recorder binary</strong> (macOS,
          Apple Silicon):
        </p>
        <CodeBlock
          code="curl -fsSL https://raw.githubusercontent.com/nitinm21/domino/main/install.sh | sh"
          copyLabel="Copy install command"
        />
      </li>
      <li className="leading-relaxed">
        <p className="mb-2.5">
          <strong className="font-semibold">
            Start Claude Code and install the plugin:
          </strong>
        </p>
        <CodeBlock
          code={`/plugin marketplace add nitinm21/domino
/plugin install domino@domino`}
          copyLabel="Copy Claude Code plugin commands"
        />
      </li>
      <li className="leading-relaxed">
        <p className="mb-2.5">
          <strong className="font-semibold">Record meetings in Claude Code:</strong>
        </p>
        <CodeBlock
          code={`/mstart
… hold the meeting …
/mstop`}
          copyLabel="Copy meeting commands"
        />
      </li>
    </ol>
  );
}

function CodexSteps() {
  return (
    <ol className="list-decimal space-y-6 pl-5 marker:font-mono marker:text-ink-faint">
      <li className="leading-relaxed">
        <p className="mb-2.5">
          <strong className="font-semibold">Install the recorder binary</strong> (macOS,
          Apple Silicon):
        </p>
        <CodeBlock
          code="curl -fsSL https://raw.githubusercontent.com/nitinm21/domino-codex/main/install.sh | sh"
          copyLabel="Copy install command"
        />
      </li>
      <li className="leading-relaxed">
        <p className="mb-2.5">
          <strong className="font-semibold">Open Codex CLI and check plugins first.</strong>{" "}
          Inside Codex CLI, enter:
        </p>
        <CodeBlock code="/plugins" copyLabel="Copy plugins command" />
        <p className="mt-3.5 mb-2.5 text-ink-muted">
          If <code>Domino</code> already appears there, install it and skip to step 3. If
          it does <strong className="font-semibold text-ink">not</strong> appear, run this
          in your regular terminal (
          <strong className="font-semibold text-ink">not inside Codex CLI</strong>):
        </p>
        <CodeBlock
          code="codex marketplace add nitinm21/domino-codex --ref stable --sparse .agents/plugins --sparse plugins/domino"
          copyLabel="Copy marketplace command"
        />
        <p className="mt-3.5 text-ink-muted">
          Then go back to Codex CLI, enter <code>/plugins</code> again, and install{" "}
          <code>Domino</code>.
        </p>
      </li>
      <li className="leading-relaxed">
        <p className="mb-2.5">
          <strong className="font-semibold">Record meetings in Codex CLI:</strong>
        </p>
        <CodeBlock
          code={`$mstart
… hold the meeting …
$mstop`}
          copyLabel="Copy meeting commands"
        />
      </li>
    </ol>
  );
}

export function Install() {
  const { tool } = useTool();
  return (
    <section id="install" className="mt-24">
      <Reveal>
        <SectionHeader title="Install" />
        <div className="mb-3">
          <ToolTabs layoutId="tool-tabs-install" />
        </div>
        <AnimatePresence mode="wait" initial={false}>
          <motion.div
            key={tool}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -6 }}
            transition={{ duration: 0.22, ease: [0.21, 0.47, 0.32, 0.98] }}
          >
            {tool === "claude-code" ? <ClaudeCodeSteps /> : <CodexSteps />}
            <p className="mt-7 text-sm text-ink-muted">
              On first{" "}
              <code>{tool === "claude-code" ? "/mstart" : "$mstart"}</code>, macOS will
              prompt for Microphone and System Audio permissions.
            </p>
          </motion.div>
        </AnimatePresence>
      </Reveal>
    </section>
  );
}
