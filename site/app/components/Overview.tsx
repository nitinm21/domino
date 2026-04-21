import { Reveal } from "./Reveal";
import { SectionHeader } from "./SectionHeader";

export function Overview() {
  return (
    <section id="overview" className="mt-24">
      <Reveal>
        <SectionHeader title="What Domino does" />
        <p className="mb-3.5 text-[17px] leading-relaxed">
          After most working conversations, someone has to sit down and translate what was said
          into edits across many places: tickets in one project, spec changes in another, code in a
          third, follow-ups for a fourth. That translation is tedious, lossy, and often skipped.
        </p>
        <p className="mb-3.5 leading-relaxed text-ink-muted">
          Domino does the fan-out automatically. It records the meeting, transcribes it locally,
          routes each segment to the project it touched, and writes a grounded implementation plan
          you can execute. Go from meeting to merge effortlessly.
        </p>
        {/*
        <blockquote className="my-5 border-l-[3px] border-ink pl-4 text-ink-muted">
          <p className="m-0">
            <strong className="font-semibold text-ink">
              Routing and synthesis are the heart of the product.
            </strong>{" "}
            Execution is a convenience, not the point. Domino is not trying to replace the human —
            it&apos;s trying to be the sounding board that has already done the boring fan-out
            work, so you show up to a structured, grounded starting place per project and decide
            what to do next.
          </p>
        </blockquote>
        */}
      </Reveal>
    </section>
  );
}
