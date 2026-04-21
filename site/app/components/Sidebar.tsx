"use client";

import { useEffect, useMemo, useState } from "react";
import { DominoMark } from "./DominoMark";

type NavItem = { id: string; href: string; label: string };
type NavGroup = { label?: string; items: NavItem[] };

const GROUPS: NavGroup[] = [
  {
    items: [
      { id: "overview", href: "#overview", label: "Overview" },
      { id: "how", href: "#how", label: "How it works" },
      { id: "install", href: "#install", label: "Install" },
      { id: "commands", href: "#commands", label: "Commands" },
    ],
  },
  {
    label: "Reference",
    items: [
      { id: "privacy", href: "#privacy", label: "Privacy" },
      { id: "requirements", href: "#requirements", label: "Requirements" },
      { id: "troubleshooting", href: "#troubleshooting", label: "Troubleshooting" },
    ],
  },
];

function useActiveSection(ids: string[]) {
  const [active, setActive] = useState<string>("");

  useEffect(() => {
    if (typeof window === "undefined") return;

    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible.length > 0) {
          setActive(visible[0].target.id);
        }
      },
      { rootMargin: "-25% 0px -65% 0px", threshold: 0 },
    );

    ids.forEach((id) => {
      const el = document.getElementById(id);
      if (el) observer.observe(el);
    });

    return () => observer.disconnect();
  }, [ids]);

  return active;
}

export function Sidebar() {
  const ids = useMemo(() => GROUPS.flatMap((g) => g.items.map((i) => i.id)), []);
  const active = useActiveSection(ids);

  return (
    <aside className="sticky top-0 hidden h-screen w-[240px] shrink-0 flex-col justify-between border-r border-rule px-8 py-10 lg:flex">
      <div>
        <a
          href="#top"
          className="mb-11 inline-flex items-center gap-2.5 text-[17px] font-semibold tracking-tight text-ink transition-opacity hover:opacity-80"
        >
          <DominoMark className="h-[18px] w-[18px] shrink-0" />
          Domino Meet
        </a>

        <nav className="space-y-9">
          {GROUPS.map((group, idx) => (
            <div key={idx}>
              {group.label ? (
                <p className="mb-4 pt-3 text-[13px] font-medium tracking-[-0.01em] text-ink-faint">
                  {group.label}
                </p>
              ) : null}
              <ul className={group.label ? "space-y-3" : "space-y-2.5"}>
                {group.items.map((item) => {
                  const isActive = active === item.id;
                  return (
                    <li key={item.href}>
                      <a
                        href={item.href}
                        className={`block text-[15px] transition-colors ${
                          isActive
                            ? "font-semibold text-ink"
                            : "text-ink-muted hover:text-ink"
                        }`}
                      >
                        {item.label}
                      </a>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </nav>
      </div>

      <div className="flex items-center gap-3 text-[13px] text-ink-faint">
        <span className="font-mono">v0.1.0</span>
        <span aria-hidden>·</span>
        <a
          href="https://github.com/nitinm21/domino"
          aria-label="GitHub"
          className="inline-flex items-center transition-colors hover:text-ink"
        >
          <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
            <path d="M8 0C3.58 0 0 3.58 0 8a8 8 0 005.47 7.59c.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z" />
          </svg>
        </a>
      </div>
    </aside>
  );
}
