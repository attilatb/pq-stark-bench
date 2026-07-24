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
    </div>
  );
}
