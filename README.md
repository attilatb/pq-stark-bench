# PQ-STARK-BENCH

Reproducible measurements of what it costs to verify a post-quantum signature
inside a general-purpose zkVM, with classical baselines measured on the same
machine.

**Dashboard:** see the deployed site. **Raw data:** [`results/`](results/).
**Survey with access dates:** [`docs/LITERATURE.md`](docs/LITERATURE.md).

## The question

Under validity-proof (rollup) settlement, signatures are verified inside the
proof circuit and never posted to the settlement layer. Only the proof is
posted. So the on-chain footprint of a transaction stops depending on the
signature scheme: it is 121 bytes per transaction whether the user signed with
Ed25519 (64-byte signature) or SLH-DSA-128f (17,088-byte signature).

That moves the entire cost of post-quantum signatures from chain storage to
prover time. Prover time for the standardized post-quantum schemes inside a
general-purpose zkVM is what this project measures.

## What is new here, stated precisely

Post-quantum signature verification inside a proof system is **not** new:

- Hash-based signatures were verified inside a STARK, with prover time, memory
  and proof size published across batch sizes 128 to 1024, in
  [ePrint 2021/1048](https://eprint.iacr.org/2021/1048) (AsiaCCS 2022).
- Falcon was implemented and profiled in Cairo by BTQ and StarkWare in
  September 2023.
- [s2morrow](https://github.com/starkware-bitcoin/s2morrow) has Falcon-512 and
  SPHINCS+ verifiers in Cairo targeting Stwo.
- [leanBench](https://github.com/leanEthereum/leanBench) continuously measures
  aggregate proving of a post-quantum hash-based signature across batch sizes.

What has not been published is a reproducible, multi-prover measurement of
prover wall-clock time and peak memory for the three NIST-standardized
signature schemes (ML-DSA-44, Falcon-512, SLH-DSA-128s) inside general-purpose
zkVMs, on named hardware, with a classical control measured in the same
harness, and with the precompile asymmetry disclosed.

Every qualifier in that sentence is load-bearing. If any of this gets published
by someone else first, this becomes an independent reproduction plus a wider
matrix, which is still worth having.

## Two things this project will not claim

**Per-signature prover cost amortizes with batch size.** It largely does not.
Prover time and memory grow close to linearly in the number of signatures, so
cost per signature is roughly flat and only proof bytes amortize. This is
already established. It is reported here, not discovered here.

**Post-quantum is N times slower than classical.** Not without a disclosure
next to it. Neither RISC Zero nor SP1 ships a lattice or NTT accelerator, while
both accelerate Ed25519 and ECDSA. Any raw ratio overstates the post-quantum
penalty, so accelerated and unaccelerated classical baselines are both
reported.

## Reproduce

```bash
git clone https://github.com/attilatb/pq-stark-bench
cd pq-stark-bench
just bench-native
```

The Rust toolchain is pinned in `rust-toolchain.toml` and the exact compiler is
recorded in every results file, along with CPU, core count, RAM, OS and target
triple. Statistics are median and p95 by nearest rank over the raw sample
vector, so every published figure is a value that was actually observed rather
than an interpolation.

## Status

| Phase | State |
|---|---|
| 1. Native benchmarks | Ed25519 and ECDSA secp256k1 measured. Post-quantum schemes in progress. |
| 2. In-circuit benchmarks (RISC Zero and SP1) | Cycle counts measured for Ed25519, Falcon-512 and ML-DSA-44 on both provers, across batch sizes. One full RISC Zero proof. Wall-clock comparison pending the x86 run. |
| 3. Dashboard | Live, rendering real measurements. |
| 4. Reference design | Not started. |

### Measured so far, in-circuit

Cycles to verify one signature, N=1, stock unless noted, Apple M3 Max:

| scheme | RISC Zero (RV32IM) | SP1 (RV64IM) |
| --- | --- | --- |
| ed25519, curve25519 precompile | 886,721 | not yet measured |
| falcon-512 | 1,055,318 | 552,817 |
| ed25519 (stock) | 3,244,248 | 791,791 |
| ml-dsa-44 | 4,029,079 | 1,788,262 |
| slh-dsa-sha2-128s | 20,853,851 | 17,999,691 |

Both provers agree on the ordering. Three findings worth stating plainly, all
measured, none a discovery:

- Unaccelerated, Falcon-512 verification is cheaper in-circuit than Ed25519.
  The precompile is what flips it: with the RISC Zero curve25519 accelerator,
  Ed25519 drops to 886,721 cycles, a 3.66x reduction, and lands below Falcon.
  That is the precompile asymmetry made concrete, and the harness asserts the
  precompile actually engaged rather than assuming it.
- Hash-based SLH-DSA-128s is far more expensive to verify in-circuit than
  either lattice scheme, roughly 5x ML-DSA-44 and 20x accelerated Ed25519,
  because its verification is dominated by a large number of SHA-256
  invocations. That is the honest cost of the conservative hash-based option.
- Per-signature cycle cost is flat across batch sizes 1 to 16 for every scheme
  (for example ML-DSA-44 stays within 0.1 percent of 4.03M cycles per
  signature). Only proof size amortizes. This confirms prior work on our own
  numbers.

Anything not yet measured renders as "not yet measured". No figure on the site
or in this repository is estimated, extrapolated, or copied from a reference
table.

## Hard rules

- No cryptography is implemented here. Every scheme binds to a vetted
  third-party implementation.
- No number is published that did not come from an actual run, with hardware,
  versions and methodology disclosed.
- A benchmark that fails or proves infeasible is published as a negative
  result.
- Language is "post-quantum" or "quantum-resistant per NIST standardization", never "quantum-proof" or "unbreakable". <!-- copy-rules:allow -->
  Falcon has no published FIPS draft as of July 2026, so it is described as
  NIST selected rather than FIPS conformant.

`just check-copy` enforces the wording rules mechanically in CI.

## Layout

```
crates/tx-format/     canonical transaction byte accounting
crates/bench-native/  native sign/verify/keygen harness
results/native/       raw measurement files, one per run
site/                 dashboard (Vite, React, Tailwind, Recharts)
docs/LITERATURE.md    prior-art survey with access dates
docs/METHODOLOGY.md   what is measured, and what the numbers do not show
docs/research/        raw research transcripts behind the survey
```

## License

Apache-2.0.
