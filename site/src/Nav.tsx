// Minimal path-based navigation. Netlify serves index.html for any path (the
// SPA redirect in netlify.toml), so /technical-note resolves to this app and
// the router below picks the view. Links use pushState to avoid a reload.

import { useEffect, useState } from "react";

export function useRoute(): string {
  const [path, setPath] = useState(
    typeof window !== "undefined" ? window.location.pathname : "/"
  );
  useEffect(() => {
    const onPop = () => setPath(window.location.pathname);
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);
  return path;
}

export function navigate(to: string) {
  if (window.location.pathname === to) return;
  window.history.pushState({}, "", to);
  window.dispatchEvent(new PopStateEvent("popstate"));
  window.scrollTo(0, 0);
}

function NavLink({
  to,
  active,
  children,
}: {
  to: string;
  active: boolean;
  children: React.ReactNode;
}) {
  return (
    <a
      href={to}
      onClick={(e) => {
        e.preventDefault();
        navigate(to);
      }}
      className={`rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
        active
          ? "bg-[var(--color-panel-2)] text-[var(--color-fg)]"
          : "text-[var(--color-muted)] hover:text-[var(--color-fg)]"
      }`}
    >
      {children}
    </a>
  );
}

export function Nav({ path }: { path: string }) {
  const onNote = path.startsWith("/technical-note");
  return (
    <nav className="sticky top-0 z-20 border-b border-[var(--color-line)] bg-[var(--color-ink)]/90 backdrop-blur">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-5 py-3">
        <a
          href="/"
          onClick={(e) => {
            e.preventDefault();
            navigate("/");
          }}
          className="text-sm font-semibold tracking-tight"
        >
          PQ-STARK-BENCH
        </a>
        <div className="flex items-center gap-1">
          <NavLink to="/" active={!onNote}>
            Benchmark
          </NavLink>
          <NavLink to="/technical-note" active={onNote}>
            Technical note
          </NavLink>
          <a
            href="https://github.com/attilatb/pq-stark-bench"
            target="_blank"
            rel="noopener noreferrer"
            className="ml-1 rounded-md px-3 py-1.5 text-xs font-medium text-[var(--color-muted)] hover:text-[var(--color-fg)]"
          >
            GitHub
          </a>
        </div>
      </div>
    </nav>
  );
}
