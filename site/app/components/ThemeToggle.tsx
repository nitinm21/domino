"use client";

import { useEffect, useState } from "react";

type Mode = "light" | "dark";

const STORAGE_KEY = "domino-theme";

function readSystemMode(): Mode {
  return typeof window !== "undefined" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function readInitialMode(): Mode {
  if (typeof window === "undefined") return "light";
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark") return stored;
  } catch {
    /* ignore */
  }
  return readSystemMode();
}

function SunIcon({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className={className}
    >
      <circle cx="12" cy="12" r="4" />
      <path d="M12 3v1.8" />
      <path d="M12 19.2V21" />
      <path d="M3 12h1.8" />
      <path d="M19.2 12H21" />
      <path d="M5.64 5.64l1.27 1.27" />
      <path d="M17.09 17.09l1.27 1.27" />
      <path d="M5.64 18.36l1.27-1.27" />
      <path d="M17.09 6.91l1.27-1.27" />
    </svg>
  );
}

function MoonIcon({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
      className={className}
    >
      <path d="M20.3 14.9a8 8 0 0 1-10.96-9.77.7.7 0 0 0-.93-.87A9.5 9.5 0 1 0 21.18 15.85a.7.7 0 0 0-.88-.95Z" />
    </svg>
  );
}

export function ThemeToggle({ className }: { className?: string }) {
  const [mounted, setMounted] = useState(false);
  const [mode, setMode] = useState<Mode>("light");

  useEffect(() => {
    setMode(readInitialMode());
    setMounted(true);
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => {
      let hasExplicit = false;
      try {
        const stored = window.localStorage.getItem(STORAGE_KEY);
        hasExplicit = stored === "light" || stored === "dark";
      } catch {
        /* ignore */
      }
      if (hasExplicit) return;
      const next: Mode = mql.matches ? "dark" : "light";
      setMode(next);
      document.documentElement.dataset.theme = next;
    };
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, []);

  function toggle() {
    const next: Mode = mode === "dark" ? "light" : "dark";
    setMode(next);
    document.documentElement.dataset.theme = next;
    try {
      window.localStorage.setItem(STORAGE_KEY, next);
    } catch {
      /* ignore */
    }
  }

  const label =
    mode === "dark" ? "Switch to light mode" : "Switch to dark mode";

  return (
    <button
      type="button"
      onClick={toggle}
      aria-label={label}
      title={label}
      className={`inline-flex h-9 w-9 items-center justify-center rounded-md border border-rule bg-paper-soft text-ink-muted transition-colors hover:border-rule-strong hover:text-ink focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/50 ${
        className ?? ""
      }`.trim()}
    >
      <span
        aria-hidden
        className={`transition-opacity duration-150 ${
          mounted ? "opacity-100" : "opacity-0"
        }`}
      >
        {mode === "dark" ? (
          <MoonIcon className="h-[15px] w-[15px]" />
        ) : (
          <SunIcon className="h-[16px] w-[16px]" />
        )}
      </span>
    </button>
  );
}
