import { Reveal } from "./Reveal";
import { SectionHeader } from "./SectionHeader";

export function Overview() {
  return (
    <section id="overview" className="mt-24">
      <Reveal>
        <SectionHeader title="What Domino does" />
        <p className="mb-3.5 text-[17px] leading-relaxed">
          You finish an engineering meeting where ten things changed at once: the API shape is
          different, one edge case needs a fix, a migration has to happen before release, and
          somebody needs to update the docs so the rest of the team does not build against stale
          assumptions. Everyone leaves aligned, but the real work is still trapped inside the
          conversation until someone sits down and translates it into code changes, tickets, and
          follow-ups.
        </p>
        <p className="mb-3.5 leading-relaxed text-ink-muted">
          Domino does that translation for you. It records the meeting, transcribes it locally,
          separates the decisions by topic, routes each segment to the codebase or project it
          touched, and writes a grounded implementation plan for what should happen next. Instead
          of relying on memory and scattered notes, you leave the meeting with work that is already
          structured and ready to execute.
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
