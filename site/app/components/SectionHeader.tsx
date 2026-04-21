type SectionHeaderProps = {
  title: string;
  className?: string;
};

export function SectionHeader({ title, className }: SectionHeaderProps) {
  return (
    <div className={`section-header ${className ?? ""}`.trim()}>
      <h2 className="section-title">{title}</h2>
      <div className="section-rule" aria-hidden />
    </div>
  );
}
