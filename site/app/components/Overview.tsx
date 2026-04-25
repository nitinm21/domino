import { Reveal } from "./Reveal";
import { SectionHeader } from "./SectionHeader";
import { VideoEmbed } from "./VideoEmbed";

export function Overview() {
  return (
    <section id="overview" className="mt-24">
      <Reveal>
        <SectionHeader title="What Domino does" />
        <p className="mb-3.5 text-[17px] leading-relaxed">
          You finish a meeting where ten things changed at once: the API shape is different, one
          edge case needs a fix, a migration has to happen before release, and somebody needs to
          update the docs so the rest of the team does not build against stale assumptions.
          Everyone leaves aligned, but the real work is still trapped inside the conversation until
          someone sits down and translates it into a plan your agent can execute.
        </p>
        <p className="mb-3.5 leading-relaxed text-ink-muted">
          Domino does that for you. It records the meeting and transcribes it locally. With its
          understanding of the codebase, it writes a grounded implementation plan you can execute.
          Instead of relying on memory and scattered notes, you leave the meeting with work that is
          already structured and ready to execute.
        </p>
        <p className="mb-3 mt-6 text-xs text-ink-muted">
          Prefer to see it in action? Here&apos;s a short walkthrough:
        </p>
        <VideoEmbed />
        <p className="mt-3 text-xs text-ink-muted">
          <a
            href="https://nitinm21.github.io/cc-chat-transcript/"
            target="_blank"
            rel="noopener noreferrer"
            className="underline decoration-ink-muted/40 underline-offset-2 transition-colors hover:text-ink hover:decoration-ink"
          >
            Claude Code transcript from this video (via <code className="font-mono">/export</code>) →
          </a>
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
