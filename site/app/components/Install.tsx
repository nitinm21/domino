import { Reveal } from "./Reveal";
import { CodeBlock } from "./CodeBlock";
import { SectionHeader } from "./SectionHeader";

export function Install() {
  return (
    <section id="install" className="mt-24">
      <Reveal>
        <SectionHeader title="Install" />
        <div className="space-y-10">
          <div>
            <h3 className="mb-3 text-[17px] font-semibold tracking-[-0.01em]">Claude Code</h3>
            <ol className="list-decimal space-y-6 pl-5 marker:font-mono marker:text-ink-faint">
              <li className="leading-relaxed">
                <p className="mb-2.5">
                  <strong className="font-semibold">Install the recorder binary</strong> (macOS,
                  Apple Silicon):
                </p>
                <CodeBlock
                  code="curl -fsSL https://raw.githubusercontent.com/nitinm21/domino/main/install.sh | sh"
                  copyLabel="Copy install command"
                />
              </li>
              <li className="leading-relaxed">
                <p className="mb-2.5">
                  <strong className="font-semibold">Add the plugin inside Claude Code:</strong>
                </p>
                <CodeBlock
                  code={`/plugin marketplace add nitinm21/domino
/plugin install domino@domino`}
                  copyLabel="Copy Claude Code plugin commands"
                />
              </li>
              <li className="leading-relaxed">
                <p className="mb-2.5">
                  <strong className="font-semibold">Record a meeting:</strong>
                </p>
                <CodeBlock
                  code={`/mstart
… hold the meeting …
/mstop`}
                  copyLabel="Copy meeting commands"
                />
              </li>
            </ol>
            <p className="mt-7 text-sm text-ink-muted">
              On first <code>/mstart</code>, macOS will prompt for Microphone and Screen Recording
              permissions.
            </p>
          </div>

          <div className="border-t border-rule pt-8">
            <h3 className="text-[17px] font-semibold tracking-[-0.01em]">Codex</h3>
            <p className="mt-2 leading-relaxed text-ink-muted">Coming soon.</p>
          </div>
        </div>
      </Reveal>
    </section>
  );
}
