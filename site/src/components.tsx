import type { ReactNode } from "react";

export function Pill({ children }: { children: ReactNode }) {
  return (
    <span className="inline-flex items-center gap-2 rounded-full border border-[var(--color-line)] bg-[var(--color-panel-2)] px-3 py-1 text-[11px] uppercase tracking-wider text-[var(--color-muted)]">
      <span className="h-1.5 w-1.5 rounded-full bg-[var(--color-accent)]" />
      {children}
    </span>
  );
}

export function Stat({
  label,
  value,
  sub,
}: {
  label: string;
  value: string;
  sub?: string;
}) {
  const missing = value === "not yet measured";
  return (
    <div className="rounded-lg border border-[var(--color-line)] bg-[var(--color-panel)] p-4">
      <div className="text-[11px] uppercase tracking-wider text-[var(--color-muted)]">
        {label}
      </div>
      <div
        className={`mt-2 text-xl font-semibold tabular-nums ${
          missing ? "italic text-[var(--color-muted)]" : ""
        }`}
      >
        {value}
      </div>
      {sub && (
        <div className="mt-1 truncate text-[11px] text-[var(--color-muted)]">
          {sub}
        </div>
      )}
    </div>
  );
}

export function Section({
  id,
  title,
  lead,
  children,
}: {
  id: string;
  title: string;
  lead?: string;
  children: ReactNode;
}) {
  return (
    <section id={id} className="scroll-mt-8 border-b border-[var(--color-line)] py-14 last:border-0">
      <h2 className="text-xl font-bold tracking-tight sm:text-2xl">{title}</h2>
      {lead && (
        <p className="mt-3 max-w-3xl text-sm leading-relaxed text-[var(--color-muted)]">
          {lead}
        </p>
      )}
      <div className="mt-8">{children}</div>
    </section>
  );
}

export function Panel({
  children,
  className = "",
  accent = false,
  tone,
}: {
  children: ReactNode;
  className?: string;
  accent?: boolean;
  tone?: "warn";
}) {
  const border = tone === "warn"
    ? "border-[var(--color-warn)]/40"
    : accent
      ? "border-[var(--color-accent)]/40"
      : "border-[var(--color-line)]";
  return (
    <div
      className={`rounded-lg border ${border} bg-[var(--color-panel)] p-5 ${className}`}
    >
      {children}
    </div>
  );
}

export function NotMeasured({ what }: { what: string }) {
  return (
    <div className="rounded-lg border border-dashed border-[var(--color-line)] bg-[var(--color-panel)]/50 p-8 text-center">
      <div className="text-sm italic text-[var(--color-muted)]">
        not yet measured
      </div>
      <p className="mx-auto mt-2 max-w-md text-xs leading-relaxed text-[var(--color-muted)]">
        {what}
      </p>
    </div>
  );
}
