"use client";

import { useEffect, useState } from "react";

const VIDEO_ID = "7VwDPwbrdQI";
const THUMBNAIL_URL = `https://img.youtube.com/vi/${VIDEO_ID}/maxresdefault.jpg`;
const EMBED_URL = `https://www.youtube.com/embed/${VIDEO_ID}?autoplay=1&rel=0`;

export function VideoEmbed() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("keydown", onKey);
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", onKey);
      document.body.style.overflow = prevOverflow;
    };
  }, [open]);

  return (
    <>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="group relative block w-full overflow-hidden rounded-xl border border-rule bg-paper-soft transition-colors hover:border-rule-strong"
        aria-label="Play Domino demo video"
      >
        <div className="relative aspect-video w-full">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src={THUMBNAIL_URL}
            alt=""
            className="absolute inset-0 h-full w-full object-cover"
            loading="lazy"
          />
          <div className="absolute inset-0 bg-black/25 transition-colors group-hover:bg-black/15" />
          <div className="absolute inset-0 flex items-center justify-center">
            <span className="inline-flex h-20 w-20 items-center justify-center rounded-full bg-white/95 shadow-lg transition-transform group-hover:scale-105 md:h-24 md:w-24">
              <svg
                viewBox="0 0 24 24"
                className="h-9 w-9 text-black md:h-11 md:w-11"
                aria-hidden="true"
              >
                <path
                  d="M9 6.5v11l9-5.5z"
                  fill="currentColor"
                  stroke="currentColor"
                  strokeWidth="2.2"
                  strokeLinejoin="round"
                  strokeLinecap="round"
                />
              </svg>
            </span>
          </div>
          <div className="absolute bottom-3 left-3 rounded-md bg-black/70 px-2 py-1 text-xs font-medium text-white">
            Watch 5-min demo
          </div>
        </div>
      </button>

      {open ? (
        <div
          role="dialog"
          aria-modal="true"
          aria-label="Domino demo video"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4 md:p-8"
          onClick={() => setOpen(false)}
        >
          <button
            type="button"
            onClick={() => setOpen(false)}
            aria-label="Close video"
            className="absolute right-4 top-4 inline-flex h-10 w-10 items-center justify-center rounded-full bg-white/10 text-white transition-colors hover:bg-white/20"
          >
            <svg viewBox="0 0 24 24" className="h-5 w-5" aria-hidden="true">
              <path
                d="M6 6l12 12M18 6L6 18"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                fill="none"
              />
            </svg>
          </button>
          <div
            className="relative w-full max-w-5xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="relative aspect-video w-full overflow-hidden rounded-xl bg-black shadow-2xl">
              <iframe
                src={EMBED_URL}
                title="Domino demo"
                className="absolute inset-0 h-full w-full"
                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                referrerPolicy="strict-origin-when-cross-origin"
                allowFullScreen
              />
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}
