import { CodeBlock } from "./CodeBlock";
import { Reveal } from "./Reveal";
import { SectionHeader } from "./SectionHeader";

const ROWS: Array<[string, React.ReactNode]> = [
  ["/mstart", "Start recording. Captures mic and system audio."],
  [
    "/mstat",
    <>
      Show the active session, or <code>{"{}"}</code> if idle.
    </>,
  ],
  ["/mstop", "Stop, transcribe locally, synthesize plans, optionally execute on a branch."],
];

export function Commands() {
  return (
    <section id="commands" className="mt-24">
      <Reveal>
        <SectionHeader title="Commands" />
        <div className="grid grid-cols-[max-content_1fr] items-baseline gap-x-6 gap-y-2.5">
          {ROWS.map(([cmd, desc]) => (
            <div key={cmd} className="contents">
              <div>
                <code className="whitespace-nowrap">{cmd}</code>
              </div>
              <div className="text-ink-muted">{desc}</div>
            </div>
          ))}
        </div>
        <h3 className="mb-2 mt-9 text-[17px] font-semibold tracking-[-0.01em]">
          Where recordings live
        </h3>
        <p className="mb-3.5 leading-relaxed text-ink-muted">
          Every meeting gets its own directory under{" "}
          <code>~/.domino/recordings/&lt;YYYY-MM-DD-HHMM&gt;/</code> containing:
        </p>
        <CodeBlock
          code={`meeting.opus         — stereo Opus recording (L = mic, R = system)
transcript.json      — structured transcript
recorder.log         — daemon log
transcription.log    — transcription phase log
plan.md              — written after /mstop if synthesis produced actionable content`}
          copyLabel="Copy recording directory contents"
        />
      </Reveal>
    </section>
  );
}
