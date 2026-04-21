import type { Config } from "tailwindcss";

const colorVar = (name: string) => `rgb(var(${name}) / <alpha-value>)`;

const config: Config = {
  content: ["./app/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        ink: {
          DEFAULT: colorVar("--ink-rgb"),
          muted: colorVar("--ink-muted-rgb"),
          faint: colorVar("--ink-faint-rgb"),
        },
        paper: {
          DEFAULT: colorVar("--paper-rgb"),
          soft: colorVar("--paper-soft-rgb"),
          code: colorVar("--paper-code-rgb"),
        },
        rule: {
          DEFAULT: colorVar("--rule-rgb"),
          strong: colorVar("--rule-strong-rgb"),
        },
        accent: {
          DEFAULT: colorVar("--accent-rgb"),
          hover: colorVar("--accent-hover-rgb"),
        },
        merge: {
          DEFAULT: colorVar("--merge-rgb"),
        },
      },
      fontFamily: {
        sans: ["var(--font-inter)", "system-ui", "sans-serif"],
        mono: ["var(--font-mono)", "ui-monospace", "SFMono-Regular", "monospace"],
      },
      maxWidth: {
        prose: "780px",
      },
    },
  },
  plugins: [],
};

export default config;
