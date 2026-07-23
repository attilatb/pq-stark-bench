import payload from "./generated/results.json";

export type Family = "classical" | "lattice" | "hash";

export interface ResultRow {
  scheme: string;
  family: Family;
  standard: string;
  implementation: string;
  operation: "keygen" | "sign" | "verify" | "prove_batch";
  batch_size: number;
  median_ns: number;
  p95_ns: number;
  iterations: number;
  min_ns?: number;
  max_ns?: number;
  mean_ns?: number;
  warmup_iterations?: number;
  sig_bytes: number;
  pubkey_bytes: number;
  proof_bytes: number | null;
  prover_cycles: number | null;
  peak_ram_mb: number | null;
  prover?: string;
  prover_version?: string;
}

export interface TxSizeRow {
  scheme: string;
  model: "pubkey_in_tx" | "address_only" | "rollup_witness_only";
  on_chain_bytes: number;
  witness_bytes: number;
  total_bytes: number;
}

export interface Environment {
  cpu: string;
  cores: number;
  ram_gb: number;
  os: string;
  arch: string;
  rustc: string;
  target: string;
  hardware_class: string;
  library_versions: Record<string, string>;
  is_ci: boolean;
  frequency_pinned: boolean;
}

export interface ResultsFile {
  run_id: string;
  kind: "native" | "zkvm";
  schema_version: number;
  generated_at: string;
  environment: Environment;
  tx_sizes?: TxSizeRow[];
  results: ResultRow[];
  _file: string;
}

interface Payload {
  collected_at: string;
  native: ResultsFile[];
  zkvm: ResultsFile[];
}

const data = payload as unknown as Payload;

export const collectedAt = data.collected_at;
export const nativeRuns = data.native ?? [];
export const zkvmRuns = data.zkvm ?? [];

/** The most recent native run, or null when nothing has been measured yet. */
export function latestNativeRun(): ResultsFile | null {
  if (nativeRuns.length === 0) return null;
  return [...nativeRuns].sort((a, b) =>
    a.generated_at < b.generated_at ? 1 : -1
  )[0];
}

/**
 * Scheme metadata for schemes we intend to measure.
 *
 * `measured` is derived from the data, never hardcoded, so a scheme only shows
 * numbers once a real run has produced them.
 *
 * Sizes here are the specification figures used to explain why signature size
 * stops mattering under validity-proof settlement. Any size shown next to a
 * timing comes from the measured run instead.
 */
export interface SchemeSpec {
  scheme: string;
  label: string;
  family: Family;
  standard: string;
  specSigBytes: number;
  specPubkeyBytes: number;
  note?: string;
}

export const SCHEME_SPECS: SchemeSpec[] = [
  {
    scheme: "ecdsa-secp256k1",
    label: "ECDSA secp256k1",
    family: "classical",
    standard: "Classical. Not post-quantum.",
    specSigBytes: 64,
    specPubkeyBytes: 33,
    note: "What Bitcoin and Ethereum use today.",
  },
  {
    scheme: "ed25519",
    label: "Ed25519",
    family: "classical",
    standard: "RFC 8032. Classical, not post-quantum.",
    specSigBytes: 64,
    specPubkeyBytes: 32,
    note: "Modern classical baseline.",
  },
  {
    scheme: "falcon-512",
    label: "Falcon-512 (FN-DSA)",
    family: "lattice",
    standard: "NIST selected. Standard not yet published.",
    specSigBytes: 666,
    specPubkeyBytes: 897,
    note: "Smallest standardized post-quantum signature. Verification is integer only.",
  },
  {
    scheme: "ml-dsa-44",
    label: "ML-DSA-44",
    family: "lattice",
    standard: "NIST FIPS 204.",
    specSigBytes: 2420,
    specPubkeyBytes: 1312,
    note: "The default post-quantum signature standard.",
  },
  {
    scheme: "slh-dsa-128s",
    label: "SLH-DSA-128s",
    family: "hash",
    standard: "NIST FIPS 205.",
    specSigBytes: 7856,
    specPubkeyBytes: 32,
    note: "Hash based and conservative. Small keys, large signatures.",
  },
  {
    scheme: "slh-dsa-128f",
    label: "SLH-DSA-128f",
    family: "hash",
    standard: "NIST FIPS 205.",
    specSigBytes: 17088,
    specPubkeyBytes: 32,
    note: "Fast-signing variant. Largest signature in the matrix.",
  },
];

/** Rows from the latest native run, keyed by scheme and operation. */
export function nativeRow(
  scheme: string,
  operation: ResultRow["operation"]
): ResultRow | null {
  const run = latestNativeRun();
  if (!run) return null;
  return (
    run.results.find((r) => r.scheme === scheme && r.operation === operation) ??
    null
  );
}

export function hasNativeData(scheme: string): boolean {
  return nativeRow(scheme, "verify") !== null;
}

export const FAMILY_COLOR: Record<Family, string> = {
  classical: "var(--color-accent)",
  lattice: "var(--color-pq)",
  hash: "var(--color-hash)",
};

export function formatNs(ns: number | null | undefined): string {
  if (ns === null || ns === undefined) return "not yet measured";
  if (ns < 1000) return `${ns} ns`;
  if (ns < 1_000_000) return `${(ns / 1000).toFixed(2)} us`;
  return `${(ns / 1_000_000).toFixed(2)} ms`;
}

export function formatBytes(b: number): string {
  if (b < 1024) return `${b} B`;
  return `${(b / 1024).toFixed(2)} KiB`;
}
