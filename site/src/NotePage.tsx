// The technical note, pre-rendered from docs/TECHNICAL-NOTE.md at build time
// and styled to match the dark research-lab theme. Content is trusted (it is
// our own markdown, rendered with raw HTML disabled), so injecting it is safe.

import { noteHtml } from "./generated/note";

export default function NotePage() {
  return (
    <div className="mx-auto max-w-3xl px-5 py-12">
      <article
        className="note-prose"
        dangerouslySetInnerHTML={{ __html: noteHtml }}
      />
      <div className="mt-12 border-t border-[var(--color-line)] pt-6 text-xs text-[var(--color-muted)]">
        This note is generated from{" "}
        <a
          href="https://github.com/attilatb/pq-stark-bench/blob/main/docs/TECHNICAL-NOTE.md"
          target="_blank"
          rel="noopener noreferrer"
          className="text-[var(--color-accent)] hover:underline"
        >
          docs/TECHNICAL-NOTE.md
        </a>{" "}
        and stays in sync with the repository on every deploy.
      </div>
    </div>
  );
}
