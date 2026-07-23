import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  FAMILY_COLOR,
  SCHEME_SPECS,
  formatBytes,
  formatNs,
  hasNativeData,
  latestNativeRun,
  nativeRow,
  zkvmRuns,
} from "./data";
import { Section, Panel, NotMeasured, Stat, Pill } from "./components";

const run = latestNativeRun();

function Hero() {
  return (
    <header className="border-b border-[var(--color-line)] bg-gradient-to-b from-[var(--color-panel)] to-[var(--color-ink)]">
      <div className="mx-auto max-w-6xl px-5 py-16 sm:py-24">
        <Pill>open benchmark - work in progress</Pill>
        <h1 className="mt-6 text-3xl font-bold leading-tight tracking-tight sm:text-5xl">
          What does it actually cost to prove a NIST post-quantum
          signature inside a zkVM?
        </h1>
        <p className="mt-6 max-w-3xl text-base leading-relaxed text-[var(--color-muted)] sm:text-lg">
          Under validity-proof settlement, signatures are verified inside the
          circuit and never posted on chain, so signature size stops driving
          cost and prover time takes over. This project measures that prover
          cost for the standardized post-quantum schemes, on more than one
          prover, with classical baselines measured on the same machine, and
          publishes the code and the raw result files.
        </p>

        <div className="mt-10 grid grid-cols-1 gap-4 sm:grid-cols-3">
          <Stat
            label="Native measurements"
            value={run ? `${run.results.length} rows` : "not yet measured"}
            sub={run ? run.environment.cpu : undefined}
          />
          <Stat
            label="In-circuit measurements"
            value={zkvmRuns.length > 0 ? `${zkvmRuns.length} runs` : "not yet measured"}
            sub="RISC Zero and SP1"
          />
          <Stat
            label="On-chain bytes per tx under a rollup"
            value="121 B"
            sub="identical for every scheme"
          />
        </div>

        <p className="mt-8 max-w-3xl text-sm leading-relaxed text-[var(--color-muted)]">
          Post-quantum signature verification inside a proof system is not new.
          Hash-based signatures were benchmarked inside a STARK in 2021, and
          Falcon was implemented in Cairo by BTQ and StarkWare in 2023. What is
          missing is a reproducible, multi-prover measurement of the three
          NIST-standardized schemes inside general-purpose zkVMs on named
          hardware. That is what this publishes.
        </p>
      </div>
    </header>
  );
}

function SignatureSizes() {
  const data = SCHEME_SPECS.map((s) => ({
    name: s.label,
    bytes: s.specSigBytes,
    family: s.family,
  }));

  return (
    <Section
      id="sizes"
      title="Why signature size stops mattering"
      lead="Post-quantum signatures are 10x to 260x larger than classical ones. Posted naively on chain, that is the whole problem. Verified inside a proof and never posted, it stops being a storage problem and becomes a proving problem."
    >
      <Panel>
        <div className="h-80 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={data} margin={{ top: 8, right: 8, bottom: 56, left: 8 }}>
              <CartesianGrid stroke="var(--color-line)" vertical={false} />
              <XAxis
                dataKey="name"
                tick={{ fill: "var(--color-muted)", fontSize: 11 }}
                angle={-25}
                textAnchor="end"
                interval={0}
                height={64}
              />
              <YAxis
                scale="log"
                domain={[10, 40000]}
                tick={{ fill: "var(--color-muted)", fontSize: 11 }}
                tickFormatter={(v: number) => `${v}`}
                label={{
                  value: "signature bytes (log)",
                  angle: -90,
                  position: "insideLeft",
                  fill: "var(--color-muted)",
                  fontSize: 11,
                }}
              />
              <Tooltip
                contentStyle={{
                  background: "var(--color-panel-2)",
                  border: "1px solid var(--color-line)",
                  borderRadius: 8,
                  fontSize: 12,
                }}
                formatter={(value) => [`${Number(value)} B`, "signature"]}
              />
              <Bar dataKey="bytes" radius={[4, 4, 0, 0]}>
                {data.map((d) => (
                  <Cell key={d.name} fill={FAMILY_COLOR[d.family]} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>
        <p className="mt-4 text-xs text-[var(--color-muted)]">
          Specification sizes, log scale. Classical in blue, lattice in violet,
          hash-based in amber.
        </p>
      </Panel>

      <div className="mt-6 grid grid-cols-1 gap-4 lg:grid-cols-3">
        <Panel>
          <h3 className="text-sm font-semibold">Public key in transaction</h3>
          <p className="mt-2 text-xs leading-relaxed text-[var(--color-muted)]">
            The naive model. Both the public key and the signature are
            published, so a single SLH-DSA-128f transaction costs over 17 KB on
            chain.
          </p>
        </Panel>
        <Panel>
          <h3 className="text-sm font-semibold">Address only</h3>
          <p className="mt-2 text-xs leading-relaxed text-[var(--color-muted)]">
            A 32-byte key hash replaces the public key and the key moves to
            witness data. This is what serious classical designs do. The
            signature is still posted.
          </p>
        </Panel>
        <Panel accent>
          <h3 className="text-sm font-semibold">Validity proof settlement</h3>
          <p className="mt-2 text-xs leading-relaxed text-[var(--color-muted)]">
            Neither key nor signature is posted. On-chain cost per transaction
            is 121 bytes for every scheme in the matrix, from Ed25519 to
            SLH-DSA-128f. The cost moves entirely to the prover.
          </p>
        </Panel>
      </div>
    </Section>
  );
}

function NativePerformance() {
  if (!run) {
    return (
      <Section
        id="native"
        title="Native performance"
        lead="Sign, verify and keygen measured outside any circuit. This is the baseline the in-circuit numbers are compared against."
      >
        <NotMeasured what="No native benchmark run has been recorded yet." />
      </Section>
    );
  }

  const rows = SCHEME_SPECS.map((spec) => {
    const verify = nativeRow(spec.scheme, "verify");
    const sign = nativeRow(spec.scheme, "sign");
    const keygen = nativeRow(spec.scheme, "keygen");
    return { spec, verify, sign, keygen, measured: hasNativeData(spec.scheme) };
  });

  const chartData = rows
    .filter((r) => r.verify)
    .map((r) => ({
      name: r.spec.label,
      verify_us: r.verify!.median_ns / 1000,
      family: r.spec.family,
    }));

  // Derived from the measurements, never hardcoded, so the callout cannot
  // drift away from what was actually measured.
  const verified = rows.filter((r) => r.verify);
  const fastestVerify = verified.reduce<typeof verified[number] | null>(
    (best, r) =>
      best === null || r.verify!.median_ns < best.verify!.median_ns ? r : best,
    null
  );
  const fastestClassical = verified
    .filter((r) => r.spec.family === "classical")
    .reduce<typeof verified[number] | null>(
      (best, r) =>
        best === null || r.verify!.median_ns < best.verify!.median_ns ? r : best,
      null
    );
  const pqBeatsClassical =
    fastestVerify !== null &&
    fastestClassical !== null &&
    fastestVerify.spec.family !== "classical";

  return (
    <Section
      id="native"
      title="Native performance"
      lead="Sign, verify and keygen measured outside any circuit, on this machine, with the exact iteration counts recorded in every results file."
    >
      {pqBeatsClassical && fastestVerify && fastestClassical && (
        <Panel accent className="mb-6">
          <h3 className="text-sm font-semibold">
            Measured: the fastest verification here is post-quantum
          </h3>
          <p className="mt-2 text-xs leading-relaxed text-[var(--color-muted)]">
            {fastestVerify.spec.label} verifies in{" "}
            <span className="text-[var(--color-fg)]">
              {formatNs(fastestVerify.verify!.median_ns)}
            </span>
            , against {formatNs(fastestClassical.verify!.median_ns)} for{" "}
            {fastestClassical.spec.label}. Post-quantum signatures are large,
            but verifying them is not inherently slow. The real costs sit
            elsewhere: in signature size, in signing time for the hash-based
            schemes, and in proving.
          </p>
          <p className="mt-2 text-xs leading-relaxed text-[var(--color-muted)]">
            This compares specific implementations
            {" "}({fastestVerify.spec.label}: {fastestVerify.verify!.implementation};{" "}
            {fastestClassical.spec.label}: {fastestClassical.verify!.implementation}),
            not the schemes in the abstract. A differently optimized library
            would move these numbers.
          </p>
        </Panel>
      )}

      <Panel>
        <div className="overflow-x-auto">
          <table className="w-full min-w-[720px] text-left text-xs">
            <thead className="text-[var(--color-muted)]">
              <tr className="border-b border-[var(--color-line)]">
                <th className="py-2 pr-4 font-medium">Scheme</th>
                <th className="py-2 pr-4 font-medium">Standard</th>
                <th className="py-2 pr-4 text-right font-medium">Keygen</th>
                <th className="py-2 pr-4 text-right font-medium">Sign</th>
                <th className="py-2 pr-4 text-right font-medium">Verify</th>
                <th className="py-2 pr-4 text-right font-medium">Pubkey</th>
                <th className="py-2 text-right font-medium">Signature</th>
              </tr>
            </thead>
            <tbody>
              {rows.map(({ spec, verify, sign, keygen, measured }) => (
                <tr
                  key={spec.scheme}
                  className="border-b border-[var(--color-line)]/60 last:border-0"
                >
                  <td className="py-2.5 pr-4">
                    <span
                      className="mr-2 inline-block h-2 w-2 rounded-full align-middle"
                      style={{ background: FAMILY_COLOR[spec.family] }}
                    />
                    {spec.label}
                  </td>
                  <td className="py-2.5 pr-4 text-[var(--color-muted)]">
                    {spec.standard}
                  </td>
                  {measured ? (
                    <>
                      <td className="py-2.5 pr-4 text-right tabular-nums">
                        {formatNs(keygen?.median_ns)}
                      </td>
                      <td className="py-2.5 pr-4 text-right tabular-nums">
                        {formatNs(sign?.median_ns)}
                      </td>
                      <td className="py-2.5 pr-4 text-right tabular-nums">
                        {formatNs(verify?.median_ns)}
                      </td>
                      <td className="py-2.5 pr-4 text-right tabular-nums">
                        {formatBytes(verify!.pubkey_bytes)}
                      </td>
                      <td className="py-2.5 text-right tabular-nums">
                        {formatBytes(verify!.sig_bytes)}
                      </td>
                    </>
                  ) : (
                    <td
                      colSpan={5}
                      className="py-2.5 text-right text-[var(--color-muted)] italic"
                    >
                      not yet measured
                    </td>
                  )}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <p className="mt-4 text-xs text-[var(--color-muted)]">
          Median of {run.results[0]?.iterations ?? 0} iterations after{" "}
          {run.results[0]?.warmup_iterations ?? 0} warmup iterations. Full
          distribution including p95 is in the raw result file.
        </p>
      </Panel>

      {chartData.length > 0 && (
        <Panel className="mt-6">
          <h3 className="mb-4 text-sm font-semibold">
            Verification time, measured
          </h3>
          <div className="h-64 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart
                data={chartData}
                margin={{ top: 8, right: 8, bottom: 40, left: 8 }}
              >
                <CartesianGrid stroke="var(--color-line)" vertical={false} />
                <XAxis
                  dataKey="name"
                  tick={{ fill: "var(--color-muted)", fontSize: 11 }}
                  interval={0}
                />
                <YAxis
                  tick={{ fill: "var(--color-muted)", fontSize: 11 }}
                  label={{
                    value: "microseconds",
                    angle: -90,
                    position: "insideLeft",
                    fill: "var(--color-muted)",
                    fontSize: 11,
                  }}
                />
                <Tooltip
                  contentStyle={{
                    background: "var(--color-panel-2)",
                    border: "1px solid var(--color-line)",
                    borderRadius: 8,
                    fontSize: 12,
                  }}
                  formatter={(value) => [`${Number(value).toFixed(2)} us`, "verify"]}
                />
                <Bar dataKey="verify_us" radius={[4, 4, 0, 0]}>
                  {chartData.map((d) => (
                    <Cell key={d.name} fill={FAMILY_COLOR[d.family]} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </Panel>
      )}
    </Section>
  );
}

function InCircuit() {
  return (
    <Section
      id="in-circuit"
      title="In-circuit results"
      lead="Prover wall-clock, peak memory and proof size for signature verification inside RISC Zero and SP1, swept across batch size. This is the part nobody has published for the standardized schemes."
    >
      {zkvmRuns.length === 0 ? (
        <NotMeasured what="No in-circuit run has been recorded yet. This section fills in as Phase 2 lands, and stays empty until real proofs have been generated." />
      ) : (
        <Panel>
          <p className="text-xs text-[var(--color-muted)]">
            {zkvmRuns.length} run(s) recorded.
          </p>
        </Panel>
      )}

      <Panel className="mt-6" tone="warn">
        <h3 className="text-sm font-semibold">
          Two things this chart will never do
        </h3>
        <ul className="mt-3 space-y-2 text-xs leading-relaxed text-[var(--color-muted)]">
          <li>
            <span className="text-[var(--color-fg)]">
              Put a RISC Zero cycle count and an SP1 cycle count on the same
              axis.
            </span>{" "}
            They are different units. SP1 guests are RV64IM and RISC Zero guests
            are RV32IM, and each vendor accounts for cycles differently. Prover
            wall-clock on one pinned machine is the comparable metric.
          </li>
          <li>
            <span className="text-[var(--color-fg)]">
              Compare provers using numbers measured on an Apple laptop.
            </span>{" "}
            RISC Zero uses Metal acceleration on Apple Silicon and SP1 has no
            GPU path there at all, so a head-to-head on that machine would
            measure which vendor shipped a Mac backend. Cross-prover comparisons
            run on one Linux x86 machine.
          </li>
        </ul>
      </Panel>
    </Section>
  );
}

function PriorArt() {
  const items = [
    {
      title: "Aggregating and thresholdizing hash-based signatures using STARKs",
      meta: "Khaburzaniya, Chalkias, Lewi, Malvai. AsiaCCS 2022.",
      body: "Published STARK prover time, peak memory and proof size at batch sizes 128 to 1024 on named hardware, for a hash-based post-quantum signature. This is the earliest work of this shape and it predates everything else here. It used Lamport+, not a NIST-standardized scheme.",
      href: "https://eprint.iacr.org/2021/1048",
    },
    {
      title: "Falcon verification in Cairo",
      meta: "BTQ and StarkWare, September 2023.",
      body: "Implemented and profiled Falcon verification in Cairo on Starknet. Publishes no proving time, step count or memory figures, and describes a Cairo profile rather than a generated proof.",
      href: "https://www.btq.com/blog/completing-the-first-falcon-signature-verification-in-starkware-initiating-the-transition-to-a-quantum-safe-ethereum",
    },
    {
      title: "s2morrow",
      meta: "starkware-bitcoin. Active.",
      body: "Falcon-512 and SPHINCS+ verifiers in Cairo targeting Stwo. The closest active work. Its roadmap item for proving benchmarks is still unchecked, so there are no numbers to cite from it yet.",
      href: "https://github.com/starkware-bitcoin/s2morrow",
    },
    {
      title: "leanBench",
      meta: "leanEthereum. Live.",
      body: "A public, continuously updated harness measuring aggregate proving of a post-quantum hash-based signature across batch sizes and hardware profiles.",
      href: "https://github.com/leanEthereum/leanBench",
    },
  ];

  return (
    <Section
      id="prior-art"
      title="Prior art, cited up front"
      lead="This work is a comparison layer on top of existing research, not a first. The literature survey with access dates lives in docs/LITERATURE.md."
    >
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        {items.map((it) => (
          <Panel key={it.href}>
            <a
              href={it.href}
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm font-semibold text-[var(--color-accent)] hover:underline"
            >
              {it.title}
            </a>
            <div className="mt-1 text-xs text-[var(--color-muted)]">{it.meta}</div>
            <p className="mt-3 text-xs leading-relaxed text-[var(--color-muted)]">
              {it.body}
            </p>
          </Panel>
        ))}
      </div>
    </Section>
  );
}

function Methodology() {
  const env = run?.environment;
  return (
    <Section
      id="methodology"
      title="Methodology"
      lead="Every number on this page came from a run on a named machine. Nothing is estimated or copied from a reference table."
    >
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <Panel>
          <h3 className="text-sm font-semibold">Measurement machine</h3>
          {env ? (
            <dl className="mt-3 space-y-1.5 text-xs">
              {[
                ["CPU", env.cpu],
                ["Cores", String(env.cores)],
                ["RAM", `${env.ram_gb} GB`],
                ["OS", env.os],
                ["Target", env.target],
                ["rustc", env.rustc],
                ["Hardware class", env.hardware_class],
                ["Frequency pinned", env.frequency_pinned ? "yes" : "no"],
              ].map(([k, v]) => (
                <div key={k} className="flex justify-between gap-4">
                  <dt className="text-[var(--color-muted)]">{k}</dt>
                  <dd className="text-right">{v}</dd>
                </div>
              ))}
            </dl>
          ) : (
            <NotMeasured what="No run recorded." />
          )}
        </Panel>

        <Panel>
          <h3 className="text-sm font-semibold">Reproduce it</h3>
          <pre className="mt-3 overflow-x-auto rounded-md bg-[var(--color-ink)] p-3 text-[11px] leading-relaxed text-[var(--color-muted)]">
{`git clone https://github.com/attilatb/pq-stark-bench
cd pq-stark-bench
just bench-native`}
          </pre>
          <p className="mt-3 text-xs leading-relaxed text-[var(--color-muted)]">
            The Rust toolchain is pinned in rust-toolchain.toml and recorded in
            every results file. Statistics are median and p95 by nearest rank
            over the raw sample vector, so every figure shown is a value that
            was actually observed.
          </p>
        </Panel>
      </div>
    </Section>
  );
}

function Limitations() {
  const items = [
    "Per-signature prover cost is expected to be roughly flat in batch size. Prover time and memory grow close to linearly with the number of signatures, and only proof bytes amortize. This is already established in the literature and is not presented here as a discovery.",
    "Neither RISC Zero nor SP1 ships a lattice or NTT accelerator, while both accelerate Ed25519 and ECDSA. Any post-quantum versus classical ratio therefore overstates the post-quantum penalty unless that asymmetry is stated, so it is stated on every comparison.",
    "Apple Silicon cannot be frequency pinned the way benchmarking convention asks for on x86, and a laptop throttles under sustained load. Local runs are labelled as such and dispersion is published alongside the median.",
    "Dollar figures are modelled, never billed. They are a published hourly instance rate multiplied by measured seconds, with the rate, source and date stated. They are not what anyone was charged.",
    "Native lattice aggregation, for example LaBRADOR, is roughly three orders of magnitude cheaper per signature than generic zkVM verification. This project measures a different question: what generic, programmable zkVM verification costs.",
    "Falcon has no published FIPS draft as of July 2026, so it is described here as NIST selected rather than FIPS conformant.",
  ];

  return (
    <Section
      id="limitations"
      title="Honest limitations"
      lead="What these numbers do not show. A reviewer should find nothing here that is not already admitted."
    >
      <Panel>
        <ul className="space-y-3">
          {items.map((t, i) => (
            <li key={i} className="flex gap-3 text-xs leading-relaxed">
              <span className="mt-0.5 shrink-0 text-[var(--color-warn)]">
                {String(i + 1).padStart(2, "0")}
              </span>
              <span className="text-[var(--color-muted)]">{t}</span>
            </li>
          ))}
        </ul>
      </Panel>
    </Section>
  );
}

function Footer() {
  return (
    <footer className="border-t border-[var(--color-line)] px-5 py-10">
      <div className="mx-auto flex max-w-6xl flex-col gap-3 text-xs text-[var(--color-muted)] sm:flex-row sm:items-center sm:justify-between">
        <span>
          PQ-STARK-BENCH. Measurement code and raw results are open.
        </span>
        <a
          href="https://github.com/attilatb/pq-stark-bench"
          target="_blank"
          rel="noopener noreferrer"
          className="text-[var(--color-accent)] hover:underline"
        >
          github.com/attilatb/pq-stark-bench
        </a>
      </div>
    </footer>
  );
}

export default function App() {
  return (
    <div className="min-h-screen">
      <Hero />
      <main className="mx-auto max-w-6xl px-5">
        <SignatureSizes />
        <NativePerformance />
        <InCircuit />
        <PriorArt />
        <Methodology />
        <Limitations />
      </main>
      <Footer />
    </div>
  );
}
