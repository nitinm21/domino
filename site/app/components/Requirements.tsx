import { Reveal } from "./Reveal";
import { SectionHeader } from "./SectionHeader";

export function Requirements() {
  return (
    <section id="requirements" className="mt-24">
      <Reveal>
        <SectionHeader title="Requirements" />
        <ul className="m-0 list-none p-0">
          <li className="bullet">macOS 14+ on Apple Silicon.</li>
          <li className="bullet">Claude Code or Codex CLI.</li>
        </ul>
      </Reveal>
    </section>
  );
}
