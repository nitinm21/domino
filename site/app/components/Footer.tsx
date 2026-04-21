export function Footer() {
  return (
    <footer className="border-t border-rule">
      <div className="mx-auto flex max-w-prose flex-wrap items-center justify-end gap-4 px-7 py-7 text-sm text-ink-muted">
        <span className="inline-flex items-center gap-3">
          <a href="https://github.com/nitinm21/domino" className="transition-colors hover:text-ink">
            GitHub
          </a>
          <span aria-hidden className="text-ink-faint">
            ·
          </span>
          <a
            href="https://github.com/nitinm21/domino/blob/main/LICENSE"
            className="transition-colors hover:text-ink"
          >
            License
          </a>
          <span aria-hidden className="text-ink-faint">
            ·
          </span>
          <a
            href="https://github.com/nitinm21/domino/issues"
            className="transition-colors hover:text-ink"
          >
            Issues
          </a>
        </span>
      </div>
    </footer>
  );
}
