"use client";

import { motion } from "framer-motion";
import Image from "next/image";
import { useTool, type Tool } from "./ToolContext";

function ClaudeMark({ className }: { className?: string }) {
  return (
    <Image
      src="/claude-ai-icon.png"
      alt=""
      width={16}
      height={16}
      className={className}
      aria-hidden="true"
      unoptimized
    />
  );
}

function OpenAIMark({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      className={className}
      aria-hidden="true"
    >
      <path d="M22.282 9.821a5.985 5.985 0 0 0-.516-4.911 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.182a5.985 5.985 0 0 0-3.998 2.9 6.046 6.046 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.998-2.9 6.056 6.056 0 0 0-.748-7.073Zm-9.022 12.608a4.476 4.476 0 0 1-2.876-1.04l.142-.081 4.778-2.758a.795.795 0 0 0 .393-.682v-6.737l2.02 1.169a.071.071 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.495 4.494Zm-9.66-4.125a4.471 4.471 0 0 1-.535-3.014l.142.085 4.783 2.758a.771.771 0 0 0 .781 0l5.843-3.369v2.333a.08.08 0 0 1-.034.061l-4.833 2.79a4.499 4.499 0 0 1-6.147-1.644Zm-1.257-10.42A4.485 4.485 0 0 1 4.707 5.9v5.701a.766.766 0 0 0 .388.676l5.815 3.355-2.02 1.168a.076.076 0 0 1-.071 0l-4.83-2.787A4.504 4.504 0 0 1 2.343 7.884Zm16.596 3.855-5.833-3.374 2.015-1.164a.076.076 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.677 8.105v-5.678a.79.79 0 0 0-.406-.68Zm2.01-3.023-.141-.086-4.773-2.781a.776.776 0 0 0-.786 0L9.41 9.23V6.898a.066.066 0 0 1 .028-.062l4.83-2.787a4.499 4.499 0 0 1 6.68 4.66ZM8.307 12.864l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075A4.499 4.499 0 0 1 13.625 2.62l-.142.081-4.778 2.758a.795.795 0 0 0-.393.681v6.724Zm1.098-2.366 2.602-1.5 2.607 1.5v2.999l-2.598 1.5-2.606-1.5v-3Z" />
    </svg>
  );
}

type ToolOption = {
  id: Tool;
  label: string;
  Mark: React.ComponentType<{ className?: string }>;
  markClassName?: string;
};

const TOOLS: ToolOption[] = [
  {
    id: "claude-code",
    label: "Claude Code",
    Mark: ClaudeMark,
    markClassName: "h-[16px] w-[16px]",
  },
  { id: "codex", label: "Codex CLI", Mark: OpenAIMark },
];

type ToolTabsProps = {
  ariaLabel?: string;
  layoutId: string;
  className?: string;
};

export function ToolTabs({ ariaLabel, layoutId, className }: ToolTabsProps) {
  const { tool, setTool } = useTool();
  return (
    <div
      role="tablist"
      aria-label={ariaLabel ?? "Select your coding agent"}
      className={`grid w-full grid-cols-2 gap-1 rounded-[12px] border border-rule bg-paper-soft p-1 ${className ?? ""}`.trim()}
    >
      {TOOLS.map(({ id, label, Mark, markClassName }) => {
        const active = tool === id;
        return (
          <button
            key={id}
            role="tab"
            type="button"
            aria-selected={active}
            onClick={() => setTool(id)}
            className={`relative inline-flex items-center justify-center gap-2 rounded-[10px] px-3 py-1.5 text-[13px] font-medium transition-colors ${
              active ? "text-ink" : "text-ink-faint hover:text-ink-muted"
            }`}
          >
            {active && (
              <motion.span
                layoutId={layoutId}
                aria-hidden
                className="absolute inset-0 rounded-[10px] bg-paper shadow-sm ring-1 ring-rule-strong"
                transition={{ type: "spring", stiffness: 500, damping: 40 }}
              />
            )}
            <Mark
              className={`relative h-[14px] w-[14px] ${markClassName ?? ""}`.trim()}
            />
            <span className="relative">{label}</span>
          </button>
        );
      })}
    </div>
  );
}
