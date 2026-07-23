# Working agreements

Read `KICKOFF.md` for the full brief. This file is the operational contract.

## Session order

1. Re-run the literature search before any session that will publish claims.
   Queries: "Falcon verification zero-knowledge", "Dilithium zkVM benchmark",
   "post-quantum signature STARK aggregation", "s2morrow". Log findings with
   dates in `docs/LITERATURE.md`.
2. Scaffold or extend code.
3. Measure.
4. Update the dashboard.
5. End with: committed and pushed, site deploy green, and a three-line
   plain-language summary.

## Non-negotiable

- **Never fabricate a number.** Missing data renders as "not yet measured".
- **No custom cryptography.** Bind to vetted implementations only.
- **No `unsafe`** without written justification. `tx-format` and `bench-native`
  both set `#![forbid(unsafe_code)]`.
- **Dependencies** from crates.io with meaningful download counts only.
- **Hyphens, never em dashes**, in all copy. Enforced by `just check-copy`.

## Positioning rules

These came out of the July 2026 literature check and must not regress.

- Never claim a bare "first". Post-quantum signature verification inside a
  STARK was published in 2021 (ePrint 2021/1048) and implemented in Cairo by
  BTQ and StarkWare in September 2023.
- Scope every novelty claim exactly: standards-conformant schemes,
  general-purpose zkVM, multi-prover, reproducible, classical control in the
  same harness.
- Cite prior art prominently. Acknowledging s2morrow, leanBench and
  Dilithium-ZK is what makes this credible.
- Being scooped is fine. If someone publishes first, this becomes an
  independent reproduction plus a wider matrix. Frame it that way.

Banned phrases, enforced in CI: "world's first", "first ever", "quantum-proof",
"unbreakable", and any sentence putting an SP1 cycle count and a RISC Zero
cycle count in the same comparison.

## Measurement rules

- **Cross-prover charts run on one Linux x86 machine.** RISC Zero uses Metal on
  Apple Silicon and SP1 has no GPU path there, so a Mac head-to-head measures
  which vendor shipped a Mac backend. Mac results are published as
  single-prover only.
- **RISC Zero and SP1 cycles are different units.** SP1 guests are RV64IM,
  RISC Zero guests are RV32IM. Never share an axis. Prover wall-clock on one
  pinned machine is the comparable metric.
- **Never mix hardware classes in one chart.** Every results file carries
  `hardware_class` (`local-m3max`, `ci-x86`, `cloud-x86-cuda`). Different
  classes are separate labelled series.
- **Batch size is a parameter, never hardcoded.** Small batches locally, large
  batches in the cloud. A batch that runs out of memory is published as a
  measured wall with the batch size and the RAM figure, not dropped.
- **Disclose the precompile asymmetry** on every post-quantum versus classical
  comparison. Neither prover accelerates lattice or NTT operations; both
  accelerate Ed25519 and ECDSA.
- **Dollar figures are modelled, never billed.** Measured seconds times a
  published hourly rate, with source and date. Bonsai was shut down in
  December 2025, so no cost figure may be sourced from it.

## Facts worth not re-deriving

- Falcon verification is integer only. Floating point is confined to signing
  and key generation, so sign outside the guest and verify inside it.
- FIPS 206 has no published draft as of 2026-07-23. Say "NIST selected".
- `pqcrypto-*` crates wrap PQClean C code and have no evidence of building in a
  zkVM guest. Use pure-Rust: `fips204`, `fips205`, `fn-dsa-vrfy`.
- Per-signature prover cost is roughly flat in batch size. Only proof bytes
  amortize.

## Infrastructure

The GitHub repo and the Netlify site for this project are **new and dedicated**.
Never point a build at, rename, or redeploy any other repo or site on these
accounts.
