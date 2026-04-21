import { Reveal } from "./Reveal";
import { SectionHeader } from "./SectionHeader";

export function Requirements() {
  return (
    <section id="requirements" className="mt-24">
      <Reveal>
        <SectionHeader title="Requirements" />
        <ul className="m-0 list-none p-0">
          <li className="bullet">macOS 14+ on Apple Silicon.</li>
          <li className="bullet">
            A{" "}
            <a
              href="https://claude.com/claude-code"
              className="text-accent underline-offset-[3px] hover:text-accent-hover hover:underline"
            >
              Claude Code
            </a>{" "}
            subscription.
          </li>
        </ul>
      </Reveal>
    </section>
  );
}
