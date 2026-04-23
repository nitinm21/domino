type DominoMarkProps = {
  className?: string;
};

export function DominoMark({ className }: DominoMarkProps) {
  return (
    <svg
      viewBox="0 0 32 32"
      aria-hidden="true"
      className={className ?? "h-[18px] w-[18px] shrink-0"}
    >
      <rect width="32" height="32" rx="7" className="fill-ink" />
      <circle cx="11" cy="16" r="3" className="fill-paper" />
      <circle cx="21" cy="16" r="3" className="fill-paper" />
    </svg>
  );
}
