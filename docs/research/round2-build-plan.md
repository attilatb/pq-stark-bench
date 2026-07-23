Verified independently from the published crate source. Here is the plan.

---

# PQ-STARK-BENCH BUILD PLAN

## 0. Two new findings from this round that change the plan

**F1 (blocking, ranked risk #1). Neither prover's SHAKE patch reaches the FIPS crates.**
`ml-dsa` 0.1.1 depends on `shake = { version = "0.1", default-features = false }` (RustCrypto/XOFs), **not** `sha3`. Verified verbatim from the published crate source at `https://docs.rs/crate/ml-dsa/0.1.1/source/Cargo.toml.orig`, accessed 2026-07-23. `slh-dsa` 0.2.0-rc.5 depends on `sha2 = "0.11"` and `shake = "0.1"`.

SP1 v6.3.1 patches `sha3` (0.10.8, 0.11.0), `tiny-keccak` 2.0.2, `sha2` (0.9.9 / 0.10.6 / 0.10.8 / 0.10.9). RISC Zero v3.0.6 patches `tiny-keccak` 2.0.2 and `sha2` (0.9.9 / 0.10.6 / 0.10.7 / 0.10.8). **Neither patches a crate named `shake`, and neither patches `sha2` 0.11.** A naive `[patch.crates-io] sha3 = ...` will build cleanly, do nothing, and hand you an inflated PQ penalty. This is the single most likely way this benchmark ships a wrong headline number.

**F2 (enabling). Falcon verification is integer-only, so it is safe in a soft-float guest.**
`rust-fn-dsa` README, accessed 2026-07-23, verbatim: *"Key pair generation and signature verification use only integer operations."* Signing uses `f64` on x86_64/aarch64, but signing is host-side only. `fn-dsa-vrfy` is a separate crate, so the guest never links the float path.

---

## 1. Toolchain, exact commands

Local target: M3 Max, aarch64 macOS, 16 cores, 64 GB.

```bash
xcode-select --install
brew install protobuf cmake
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# RISC Zero  (v3.0.6, published 2026-07-17)
curl -L https://risczero.com/install | bash
rzup install cargo-risczero 3.0.6
rzup install r0vm 3.0.6
rzup install cpp

# SP1  (v6.3.1, published 2026-06-25)
curl -L https://sp1up.succinct.xyz | bash
sp1up
sp1up --c-toolchain          # installs riscv64-unknown-elf-gcc
```

Record verbatim into every results file: `cargo risczero --version`, `r0vm --version`, `cargo prove --version`, `RUSTUP_TOOLCHAIN=succinct cargo --version`, `rustc --version`.

**Will not work locally, with fallback:**

| Blocked | Why | Fallback |
|---|---|---|
| RISC Zero Groth16 | Docs: *"The Groth16 prover currently only works on x86 architecture, and so Apple Silicon is currently unsupported (even via Docker)."* | STARK succinct receipts locally; Groth16 on an x86_64 Linux box. Mark the on-chain proof-size column "not measured on this machine". |
| SP1 GPU | CUDA is Linux x86_64 only; AVX is Intel only | SP1 runs scalar CPU on the Mac. See risk #2: this makes local SP1-vs-RISC-Zero wall clock meaningless. |
| SP1 PLONK wrap | Docs: 64 GB+, wrap step "roughly 60GB" | Skip PLONK entirely. Groth16 (14 GB wrap) fits under Docker Desktop if you raise its memory limit to 32 GB+. |
| Any GPU number from run #1 | Metal kernels JIT-compile on first use ("can take a few minutes") | Always discard iteration 1; report from iteration 2 onward. |

---

## 2. Crate selection, GO/NO-GO

| Scheme | Crate | Native | RISC Zero guest | SP1 guest |
|---|---|---|---|---|
| ML-DSA-44 (FIPS 204) | `ml-dsa` 0.1.1, Apache-2.0 OR MIT, RustCrypto/signatures, updated 2026-06-05 | GO | GO with F1 caveat | GO with F1 caveat |
| Falcon-512 / FN-DSA (FIPS 206 draft) | `fn-dsa-vrfy` 0.4.0, Unlicense, pornin/rust-fn-dsa, updated 2026-07-22 | GO | **GO, cleanest PQ leg** | **GO, cleanest PQ leg** |
| SLH-DSA-128s (FIPS 205) | `slh-dsa` 0.2.0-rc.5, Apache-2.0 OR MIT | GO | GO, prerelease | GO, prerelease |
| Ed25519 (control) | `curve25519-dalek` 4.1.3 | GO | GO, precompiled | GO, precompiled |
| secp256k1 (control) | `k256` 0.13.x | GO | GO, precompiled | GO, precompiled |

**Falcon crate choice, explicitly:** use `fn-dsa-vrfy` 0.4.0, not the top-level `fn-dsa` crate, and not any Cairo port. Reasons: it is the verify-only subcrate so the guest never pulls the `f64` signing path; verification is integer-only per the README; and it is by Thomas Pornin, who also wrote the C reference. Sign host-side with `fn-dsa-sign`. Caveat to disclose: license is Unlicense (public-domain dedication), which is permissive but not OSI-conventional; and FIPS 206 is still draft, so never write "FIPS-conformant Falcon" unqualified.

**F1 mitigations, in preference order:**
1. Build every PQ scheme **twice** and publish both: `stock` (no hash patch, honest worst case) and `accel` (hash routed to the prover's Keccak circuit). The delta is the value of a SHAKE precompile for ML-DSA, which nobody has published.
2. For `accel`: RISC Zero, vendor a thin `shake` shim crate that calls patched `tiny-keccak` 2.0.2 and patch it in by path. SP1, either the same shim over patched `sha3` 0.11.0, or fork `ml-dsa` to depend on `sha3` 0.11 directly (SP1 patches that version).
3. `slh-dsa`'s `sha2` 0.11 is unpatched on both. Either shim it down to a patched 0.10.x core or accept `stock` only, labelled.

**NO-GO fallback if the guest will not compile at all:** `ml-dsa` is edition 2024 / rust-version 1.85 and `slh-dsa` matches. SP1's succinct toolchain is documented at 1.93, so it should pass. RISC Zero's pinned guest Rust version must be checked on install day; if it is below 1.85, use `rzup install rust <version>` or vendor the crate with the edition downgraded. Do not fall back to `ml-dsa` 0.0.4, which predates FIPS-204-final.

---

## 3. Phase order, thin vertical slice first

**Slice 1: Ed25519, N=1, RISC Zero only, end to end.** Guest build, `execute()` for cycles, prove with Metal, sample peak RSS, emit results JSON, render one chart. Nothing else.

Why this pair: Ed25519 is precompiled on both provers so it is the control column you need anyway; RISC Zero on the Mac has a working GPU path and needs no Groth16 for a STARK receipt; it is the cheapest workload so the edit-run loop is seconds not hours; and it exercises 100 percent of the harness plumbing with zero PQ risk. Starting at ML-DSA-on-SP1 would combine the two riskiest legs (F1 plus scalar-CPU-only proving) and you would not know whether a bad number was the harness or the scheme.

Then, in order:
2. Falcon-512 (`fn-dsa-vrfy`), N=1, RISC Zero. First real PQ datapoint, lowest-risk PQ crate.
3. ML-DSA-44, N=1, RISC Zero, both `stock` and `accel`. This is the empty cell.
4. Same three on SP1 (exposes the RV64 vs RV32 and backend asymmetries).
5. SLH-DSA-128s, N=1, both provers.
6. Batch ladder, all schemes, both provers.
7. x86_64 Linux fairness run plus Groth16 leg.
8. Stwo/Cairo lane, only if 1 to 7 are published.

Gate between phases: a phase is done when its results JSON validates against the schema and its precompile assertion passes.

---

## 4. Results JSON, minimal additions to the base schema

Keep leanBench's shape (`run_id`, `timestamp`, `machine{}`, `toolchain{}`, `workloads[]` with raw `samples_ns` plus derived `timing{}` and `resources{}`). Add four objects per workload and one array:

```jsonc
"scheme": {
  "name": "ML-DSA-44", "family": "lattice", "spec": "FIPS-204",
  "param_set": "ML-DSA-44", "crate": "ml-dsa", "crate_version": "0.1.1",
  "hash_primitive": "SHAKE-256", "conformant": true, "prerelease": false
},
"prover": {
  "name": "risc0", "version": "3.0.6",
  "isa": "riscv32im",                    // sp1 v6 is "riscv64im"
  "backend": "metal",                    // cpu-scalar | metal | cuda | avx512
  "proof_mode": "succinct",              // core|composite|succinct|compressed|groth16
  "security_bits": { "value": 96, "kind": "claimed", "source": "<docs URL>" },
  "segment_limit_po2": 20,               // risc0 only, READ it, never assume
  "precompiles_used": ["keccak_permute"],
  "precompile_assert_passed": true
},
"batch": { "n": 8, "topology": "flat", "arity": null },
"cost": {
  "cycles": 4211337, "cycles_source": "SessionInfo::cycles()",
  "padded_cycles": 8388608, "pgu": null,
  "proof_bytes": 215040, "verify_ms": 4.4
},
"caveats": [
  "no lattice/NTT precompile on this prover",
  "SHAKE not precompile-routed: shake 0.1 is unpatched (stock variant)"
]
```

Two hard rules enforced in code, not by discipline: `cost.cycles` may never share an axis across `prover.name` (the chart builder must key the axis on prover and refuse otherwise), and every rendered chart must print the union of `caveats` plus `prover.backend` and `prover.security_bits` in its footnote.

---

## 5. Batch sizes and the negative-result plan

Ladder: N in {1, 2, 4, 8, 16, 32, 64, 128}, flat topology first, powers of two so the po2 padding story stays clean.

Do not guess the wall in advance. Measure N=1, then project linearly, which is justified because flat amortization is already established prior art (ePrint 2021/1048 Table 3; Winterfell README; ZK-ACE Table 7; leanBench). Set two abort conditions before starting a ladder:

- projected wall > 30 min for a single run, or
- projected peak RSS > 48 GB (leaves 16 GB headroom on 64 GB).

Expected ordering, to be confirmed not assumed: Ed25519 and secp256k1 cheapest (precompiled), then Falcon-512, then ML-DSA-44 `accel`, then ML-DSA-44 `stock`, then SLH-DSA-128s. Expect Ed25519 to reach N=128 comfortably and ML-DSA `stock` to hit a ceiling first.

**Negative results are published cells, not failures.** On abort, emit the workload with `"status": "aborted"`, `"abort_reason": "peak_rss_exceeded" | "wall_timeout" | "guest_oom"`, the failing N, and the last good N. "ML-DSA-44 stock exceeded 48 GB at N=64 on a 64 GB M3 Max" is a headline-grade finding and is exactly the number HAPPIER omitted (it publishes no memory figures at all). Never silently drop a failed cell; a gap in the chart with no explanation is the thing reviewers attack.

---

## 6. Risks, ranked, each with a visible mitigation

1. **Silent non-acceleration (F1).** Publishes an inflated PQ penalty. Mitigation: assert, do not assume. On SP1, `report.syscall_counts[KECCAK_PERMUTE] > 0`; on RISC Zero, assert the `accel` cycle count is materially below `stock`. A test that fails the build when an `accel` run shows zero precompile use. Surface `precompile_assert_passed` in the JSON.
2. **Cross-prover comparison is not apples to apples.** RV64IM vs RV32IM, Metal vs scalar CPU, and different soundness (RISC Zero documents 96 bits for the RISC-V prover, SP1 documents 100 conjectured for Turbo and no number for Hypercube). Mitigation: schema fields above, the chart linter, and one x86_64 Linux fairness run where both provers have comparable backends. Headline on wall-clock seconds and peak RSS, never on cycles.
3. **Guest build failure on edition 2024 / rust 1.85.** Mitigation: day-one spike before any harness work, compiling `ml-dsa` to both guest targets and recording both toolchain versions.
4. **No Groth16 on Apple Silicon (RISC Zero).** Mitigation: x86_64 Linux leg; mark the column not-measured rather than leaving it blank.
5. **Memory ceiling at large N.** Mitigation: read, pin, and record `segment_limit_po2` (bounded 13 to 24; default is not published, so read it at runtime); publish the abort per section 5.
6. **Our own numbers not reproducible.** Mitigation: pin every version and a fixed RNG seed; commit the signature fixtures as test vectors with a KAT check so CI and local prove byte-identical inputs; emit raw `samples_ns`; always discard iteration 1.
7. **Overclaiming novelty.** Mitigation: a LITERATURE.md gate that must cite ePrint 2021/1048, HAPPIER (LightSec 2025), leanBench, ZK-ACE, OpenZeppelin cairo-pq-verifiers, and starkware-bitcoin/s2morrow before anything publishes. Never write "first". The defensible claim is narrow: first published prover wall-clock and peak-RAM measurements for FIPS ML-DSA-44 in a general-purpose zkVM, on named hardware, with the precompile asymmetry disclosed.
8. **`slh-dsa` is 0.2.0-rc.5, a release candidate.** Mitigation: `"prerelease": true` in the JSON; keep it off the headline chart.

---

## 7. CI versus local

**CI (free GitHub runners, x86_64, ~4 cores, 16 GB, slow): never prove.** Run only what is deterministic and cheap:
- build both guest ELFs for both provers;
- executor-only runs (`Executor::execute()` / `client.execute()`), which need no proving, to get cycle counts and syscall counts;
- the precompile assertions from risk #1;
- KAT verification of the committed fixtures;
- results-JSON schema validation and the chart linter;
- an OpenZeppelin-style one-way ratchet: fail the build if cycle counts rise against a committed baseline;
- the em-dash grep on added diff lines.

Cycle counts are deterministic, so they are a perfect CI regression signal even on a slow shared runner. Pin SP1 to exactly v6.3.1 in CI, because `ExecutionReport`'s serde format is only stable within one SP1 version.

**Local M3 Max:** all proving, all RSS sampling, the full batch ladder, both `stock` and `accel` variants.

**Rented x86_64 Linux, one shot:** the cross-prover fairness run, RISC Zero Groth16, optionally SP1 CUDA. Record it as a distinct machine fingerprint and never merge its wall-clock numbers into the Mac series.

Do not lean on GitHub's macOS arm64 runners for proving; they are constrained and would give you a third, undocumented hardware class in the dataset.