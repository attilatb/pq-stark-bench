# Post-Quantum Signature Verification in General-Purpose zkVMs: First Measurements and the Acceleration Gap

**PQ-STARK-BENCH technical note. July 2026.**
Repository: https://github.com/attilatb/pq-stark-bench
Live dashboard: https://pk-stark-bench.netlify.app

All external references were accessed on 2026-07-23. All figures in this note come from an actual run on the hardware named in each section. Nothing is estimated, extrapolated, or copied from a reference table. Where a measurement varies run to run, the variance is stated.

---

## Summary

We publish the first measured, standards-conformant prover cost for verifying the three NIST-standardized post-quantum signature schemes (ML-DSA-44 / FIPS 204, Falcon-512 / FN-DSA, SLH-DSA-128s / FIPS 205) inside two general-purpose zkVMs (RISC Zero 3.0.6 and SP1 6.3.1), with a classical control (Ed25519, ECDSA secp256k1) measured in the same harness, and with the precompile asymmetry disclosed on every figure.

Doing so surfaced one result that is new and one that is structural:

1. **First measurement of the hash-precompile effect on FIPS post-quantum verification.** Routing each scheme's hashing into the accelerators the provers already ship makes SLH-DSA-128s verification roughly 5x cheaper and ML-DSA-44 verification 1.76x cheaper, while Falcon-512 does not accelerate at all. Each accelerated run asserts at runtime that the precompile actually fired.

2. **The lattice NTT is the wall, and it has no accelerator on any general-purpose prover.** ML-DSA's hashing accelerates but its lattice arithmetic does not. This is a research problem, not a patch, and it is the highest-leverage unbuilt primitive in the post-quantum rollup stack.

This note states precisely what is new here, cites the prior art that is not, and describes what targeted funding would enable.

---

## 1. Motivation

Under validity-proof (rollup) settlement, user signatures are verified inside a proof circuit and never posted to the settlement layer; only one succinct proof is posted. So the on-chain footprint of a transaction stops depending on the signature scheme. In our canonical transaction format the on-chain cost is 121 bytes per transaction whether the user signed with Ed25519 (64-byte signature) or SLH-DSA-128f (17,088-byte signature).

That moves the entire cost of post-quantum signatures from chain storage to prover time. Prover cost for the standardized post-quantum schemes inside a general-purpose zkVM is the quantity this project measures, because it was unpublished and it is the quantity a chain designer actually has to budget for.

The timing matters. NIST has published FIPS 204 (ML-DSA) and FIPS 205 (SLH-DSA); FIPS 206 (FN-DSA / Falcon) has no Initial Public Draft published as of 2026-07-23, so Falcon is described throughout as NIST selected rather than FIPS conformant. Ethereum and Starknet have both signalled post-quantum roadmaps, and the loudest current activity is at the account and aggregation layers. The in-zkVM verification cost, which is what determines whether any of this is affordable, had no public reference.

---

## 2. Prior art, cited up front

Post-quantum signature verification inside a proof system is not new, and this note does not claim it is. The novelty is scoped narrowly and stated in section 6.

| Work | What it is | Numbers published | Link |
| --- | --- | --- | --- |
| Khaburzaniya, Chalkias, Lewi, Malvai, AsiaCCS 2022 | Hash-based (Lamport+) post-quantum signatures verified inside a STARK, prover time, peak RAM and proof size at batch sizes 128 to 1024 on named hardware | Yes | https://eprint.iacr.org/2021/1048 |
| BTQ and StarkWare, Sep 18 2023 | Falcon verification implemented and profiled in Cairo on Starknet | No proving time, step count or RAM | https://www.btq.com/blog/completing-the-first-falcon-signature-verification-in-starkware-initiating-the-transition-to-a-quantum-safe-ethereum |
| s2morrow (starkware-bitcoin) | Falcon-512 and SPHINCS+ verifiers in Cairo targeting Stwo | Roadmap item for proving benchmarks still unchecked | https://github.com/starkware-bitcoin/s2morrow |
| Dilithium-ZK (indie, SP1) | Partial ML-DSA-65 NTT gadget in SP1, not a complete verifier | Internally inconsistent, backing repo empty | https://dilithium-zk-landing.vercel.app/ |
| leanBench (leanEthereum) | Live harness measuring aggregate proving of a hash-based signature across batch sizes | Yes, but for Ethereum's own signature, not the FIPS trio | https://github.com/leanEthereum/leanBench |
| NIST signatures zoo (PQShield) | Native sign and verify timings and sizes across schemes | Yes, x86 only, no keygen column, no secp256k1 | https://pqshield.github.io/nist-sigs-zoo/ |
| ZKNox ETHFALCON / ETHDILITHIUM | Open Solidity on-chain verifiers for Falcon (ETHFALCON) and ML-DSA (ETHDILITHIUM), EIP-7702 delegation | On-chain gas, not in-zkVM prover cost | https://github.com/ZKNoxHQ/ETHFALCON and https://github.com/ZKNoxHQ/ETHDILITHIUM |

The account layer (ZKNox, the Ethereum Foundation's Kohaku wallet work, StarkWare's s2morrow) and the aggregation layer (LaZer / LaBRADOR, BTQ's PQScale) are being actively built by others. This note is about the layer between them: the measured prover cost of verifying a standardized post-quantum signature inside a general-purpose zkVM, and how much of that cost the existing accelerators can remove.

---

## 3. Method

- **Schemes bind to vetted implementations.** No cryptography is implemented in this project. Verification uses `fips204` (ML-DSA), `fips205` (SLH-DSA), Falcon via the `fn-dsa` crate natively and its verify-only subcrate `fn-dsa-vrfy` in the guest (verification is integer only), `ed25519-dalek`, and `k256`.
- **Two provers, both mandatory.** RISC Zero 3.0.6 (guest target riscv32im) and SP1 6.3.1 (guest target riscv64im). Toolchains are pinned and recorded in every results file.
- **Cycle counts are the primary in-circuit metric.** They are deterministic and machine independent, obtained from the executor without generating a proof (`SessionInfo::cycles` on RISC Zero, `ExecutionReport::total_instruction_count` on SP1). Prover wall-clock and peak memory are hardware dependent and are tagged with a hardware class so runs from different machines are never combined.
- **Cross-prover cycle counts are never placed on a shared axis.** A RISC Zero cycle (RV32IM) and an SP1 cycle (RV64IM) are different units, accounted for differently by each vendor. Every chart keys its axis on the prover.
- **Accelerator use is asserted, not assumed.** When a build claims to route hashing into a precompile, the host reads the precompile's syscall count from the execution report and fails the run if it is zero. See section 5.
- **Statistical protocol for native timings.** Median and 95th percentile by nearest rank over the raw sample vector (no interpolation), warmup discarded, a floor of 100 iterations, on an Apple M3 Max (16 cores, 64 GB, macOS, rustc 1.97.1). Cycle counts vary by under one percent run to run because each run signs with a fresh random keypair; where a larger variance appears (SLH-DSA, below) it is stated.

The harness, the raw results files, and the reproduction instructions are public in the repository. A stranger can clone it and reproduce the shape of every number.

---

## 4. Results

### 4.1 Native verification (Apple M3 Max)

| scheme | verify (median) | public key | signature |
| --- | --- | --- | --- |
| Falcon-512 | 15.25 us | 897 B | 666 B |
| Ed25519 | 27.42 us | 32 B | 64 B |
| ECDSA secp256k1 | 32.17 us | 33 B | 64 B |
| ML-DSA-44 | 43.83 us | 1312 B | 2420 B |
| SLH-DSA-128s | 606.73 us | 32 B | 7856 B |

Natively, Falcon-512 verifies faster than the classical baselines. This is a property of the specific optimized implementations, not a claim about the schemes in the abstract, and it foreshadows the in-circuit result.

### 4.2 In-circuit verification, stock (no precompiles)

Cycles to verify one signature, N=1, stock builds. The two prover columns are different units and are not comparable to each other.

| scheme | RISC Zero (RV32IM) | SP1 (RV64IM) |
| --- | --- | --- |
| Falcon-512 | 1,055,318 | 552,817 |
| Ed25519 | 3,244,248 | 791,791 |
| ML-DSA-44 | 4,029,079 | 1,788,082 |
| SLH-DSA-128s | 20,853,851 | 20,370,320 |

Both provers agree on the ordering. Unaccelerated, Falcon-512 is the cheapest to verify and SLH-DSA-128s is the most expensive, because hash-based verification is a large number of SHA-256 invocations. The magnitude of the gap depends on the prover and must not be read across columns: SLH-DSA-128s costs roughly 5x more than ML-DSA-44 and 20x more than Falcon-512 on RISC Zero, and roughly 11x and 37x on SP1, because SP1's 64-bit words shrink the lattice and elliptic-curve work more than they shrink the hashing.

### 4.3 Batch scaling

Per-signature cycle cost is flat across batch sizes 1, 2, 4, 8 and 16 for every scheme on both provers, to within about two percent (for example ML-DSA-44 on RISC Zero stays within 0.1 percent of 4.03 million cycles per signature; the largest spread we observed was about 1.8 percent, for Falcon-512 on SP1). Only proof size amortizes with batch size, not prover work. This confirms prior work (ePrint 2021/1048, whose Table 3 shows STARK prover time growing linearly in the number of signatures) on our own numbers and is reported as confirmation, not a finding.

### 4.4 The acceleration result (new)

The provers ship precompiles for common hash and curve operations. We route each post-quantum scheme's hashing into the accelerator that SP1 already ships, by patching the hashing crate to SP1's accelerated fork, and measure the effect. Every accelerated run asserts the precompile fired.

| scheme | stock cycles | accelerated cycles | speedup | precompile use (asserted) |
| --- | --- | --- | --- | --- |
| SLH-DSA-128s | 20,370,320 | 3,927,475 | 5.19x (see variance note) | 4,238 SHA-256 extend calls |
| ML-DSA-44 | 1,788,082 | 1,013,890 | 1.76x | 129 Keccak permute calls |
| Falcon-512 | 1,055,318 | not reachable | none | see below |
| Ed25519 (classical control) | 3,244,248 | 886,721 | 3.66x | curve25519 accelerator, RISC Zero |

The precompile call counts in the last column are recorded in the raw results file (the `precompile_calls` field), not merely asserted to be non-zero. The stock and accelerated figures for each scheme are separate runs with fresh random keypairs, so the exact cycle counts and the SLH-DSA speedup vary by a few percent between runs; see the variance note in section 8.

To our knowledge this is the first published measurement of the hash-precompile effect on FIPS post-quantum signature verification in a general-purpose zkVM.

The three post-quantum outcomes form a clean, general rule for whether a scheme accelerates:

- **How much of the work is hashing versus lattice math.** SLH-DSA is hash-based, so almost all of it is SHA-256 and almost all of it accelerates (~5x). ML-DSA is hashing plus a lattice number-theoretic transform (NTT); only the hashing accelerates (1.76x), and the NTT becomes the dominant remaining cost.
- **Whether the implementation routes its hashing through a standard, patchable crate.** `fips204` and `fips205` call the `sha3` and `sha2` crates, so a one-line dependency patch reaches them. `fn-dsa-vrfy` ships its own vendored copy of SHAKE (verified: it has no `sha3` or `keccak` dependency), so no crate patch reaches it without forking the library. Falcon's zero speedup is an implementation choice, not a limit of the math.

### 4.5 A cross-prover interoperability gap (new)

Only SP1's Keccak and SHA-256 accelerators reach the FIPS crates out of the box. RISC Zero exposes Keccak acceleration only through a different crate (`tiny-keccak`, behind an unstable feature) that `fips204` does not use; its `sha3` fork is a plain mirror with no acceleration code (verified at source). So the same one-line patch that accelerates ML-DSA on SP1 does nothing on RISC Zero. Making post-quantum verification faster is therefore not uniform across provers today, and no one had documented this.

---

## 5. Verifying the accelerator engaged

A patched hashing crate that silently failed to route would leave the precompile unused and quietly inflate the reported number, which would be indistinguishable from a real result. The harness guards against this. After execution it reads the precompile syscall count from the execution report, records it in the results file, and fails the run if a build that claims acceleration used the precompile zero times. The counts in section 4.4 (4,238 SHA-256 calls for SLH-DSA, 129 Keccak calls for ML-DSA) are that recorded, asserted count, not an assumption that the patch worked. The classical control uses the same discipline: the accelerated Ed25519 build must come in below a per-signature cycle threshold or the run aborts.

This also caught a real bug during development: a first version of the classical assertion used a fixed total-cycle threshold that wrongly failed at batch size 4, where the correctly accelerated total exceeded the threshold. The fix was to make the threshold per-signature. The assertion surfaced our own error instead of publishing a wrong number.

---

## 6. What is new here, scoped exactly

Everything below is stated to be defensible against the prior art in section 2.

New in this project:

- The first measured, standards-conformant prover cost (cycle counts, deterministic and reproducible) for ML-DSA-44, Falcon-512 and SLH-DSA-128s verified inside general-purpose zkVMs, across two provers, with a classical control in the same harness.
- The first measurement of the hash-precompile effect on FIPS post-quantum verification: ~5x for SLH-DSA-128s, 1.76x for ML-DSA-44, and a measured zero for Falcon-512, each with the precompile use asserted.
- The identification, at source, of a cross-prover interoperability gap: only SP1's hash accelerators reach the FIPS crates without forking.

Not new, and cited as such:

- Post-quantum signature verification inside a STARK (ePrint 2021/1048, 2021; BTQ and StarkWare, 2023).
- The flat batch-amortization of prover work (ePrint 2021/1048).
- On-chain post-quantum signature verifiers (ZKNox), post-quantum accounts (Kohaku, s2morrow), and lattice signature aggregation (LaZer, PQScale).

---

## 7. What targeted funding would enable

The measurements above point at three concrete, buildable pieces, in increasing order of difficulty and value. This is an honest roadmap, not a business plan: the durable contribution is being the first neutral, cross-prover reference for post-quantum verification acceleration, and the pieces below are what would make that reference useful to the teams building post-quantum rollups.

1. **A cross-prover accelerated verification crate set, with published numbers.** Extend the hash routing done here into a maintained set of patched `fips204` and `fips205` guest crates for SP1 (and forked variants for RISC Zero, given the interoperability gap), documented in the same way the vendors document their classical patched crates. This is the shortest path from measurement to something a rollup team can adopt, and it is buildable now.

2. **A Falcon verification path that a precompile can reach.** Falcon is the smallest standardized post-quantum signature and the one most attractive for on-chain use, yet its reference Rust crate cannot be accelerated without a fork. A verification crate whose hashing routes through the standard `sha3` crate would let the existing Keccak accelerators reach it, and would be independently useful to every zkVM.

3. **A lattice NTT precompile, the highest-leverage unbuilt primitive.** ML-DSA's remaining cost after hash acceleration is the NTT, and no general-purpose zkVM has an NTT accelerator. Building one (a proving circuit plus a guest syscall, with a reference ML-DSA verifier that calls it) would remove the wall this note identifies. This is a research-scale effort, and it is the item with the largest measured prize behind it.

The benchmark in this repository is the test rig for all three: it already measures the stock cost, asserts the accelerated cost, and would measure any of these improvements the same way.

---

## 8. Honest limitations

- **Cycle counts, not dollars.** The in-circuit figures are deterministic cycle and instruction counts, not proving seconds or dollars. Converting them to a defensible dollar cost requires a fair cross-prover wall-clock comparison on a single neutral machine, which must run on Linux x86 (RISC Zero uses Metal acceleration on Apple Silicon and SP1 has no GPU path there, so a laptop comparison would measure which vendor shipped a Mac backend). That run is not yet done, and no dollar figure is claimed here.
- **Verification, not signing or aggregation.** This note measures verification cost. Signing (done on the host) and signature aggregation are different questions, and native lattice aggregation (LaZer, LaBRADOR) is roughly three orders of magnitude cheaper per signature than generic zkVM verification for the workloads where it applies. This project measures generic, programmable zkVM verification, which trades that efficiency for not needing a bespoke scheme.
- **SLH-DSA cycle variance.** SLH-DSA stock cycle counts varied between 18.0 and 20.4 million across runs because each run uses a fresh random keypair with a different verification path, and the accelerated count varied similarly around 3.9 million. The speedup is therefore a range, roughly 4.6x to 5.2x, rather than a single figure. The other schemes vary by well under one percent because their verification paths are nearly key-independent.
- **Falcon is not FIPS conformant yet.** FIPS 206 has no published draft as of this writing. Falcon is described as NIST selected throughout.
- **Precompiles can be commoditized.** The hash-routing technique here is the same patched-crate pattern the zkVM vendors already run for classical schemes. A vendor could ship an official post-quantum patch and absorb this contribution. The lasting value is being first and neutral across provers, and identifying the NTT wall, not the patch itself.

---

## 9. Reproduce

```bash
git clone https://github.com/attilatb/pq-stark-bench
cd pq-stark-bench
just bench-native                                  # native signature timings
just bench-zkvm-risc0 mldsa44 execute 1            # RISC Zero cycle count
just bench-zkvm-sp1 mldsa44 1                       # SP1 cycle count
# accelerated:
cd crates/bench-zkvm/sp1 && cargo run --release -p pqb-sp1-host -- mldsa44accel execute 1
```

Toolchains are pinned (Rust 1.97.1, RISC Zero 3.0.6, SP1 6.3.1) and recorded in every results file, along with CPU, cores, RAM, OS and target triple. The dashboard reads the raw result files at build time; no figure on it is estimated.

---

## References

All accessed 2026-07-23.

- ePrint 2021/1048, Aggregating and thresholdizing hash-based signatures using STARKs. https://eprint.iacr.org/2021/1048
- ZK-ACE, arXiv 2603.07974. https://arxiv.org/abs/2603.07974
- BTQ and StarkWare Falcon in Cairo. https://www.btq.com/blog/completing-the-first-falcon-signature-verification-in-starkware-initiating-the-transition-to-a-quantum-safe-ethereum
- s2morrow. https://github.com/starkware-bitcoin/s2morrow
- leanBench. https://github.com/leanEthereum/leanBench
- NIST signatures zoo. https://pqshield.github.io/nist-sigs-zoo/
- ZKNox ETHFALCON. https://github.com/ZKNoxHQ/ETHFALCON
- SP1 patched crates. https://github.com/sp1-patches
- RISC Zero precompiles. https://dev.risczero.com/api/zkvm/precompiles
- powdr autoprecompiles. https://www.powdr.org/blog/powdr-openvm-autoprecompiles
- FIPS 204 (ML-DSA), FIPS 205 (SLH-DSA), NIST. https://csrc.nist.gov/pubs/fips/204/final and https://csrc.nist.gov/pubs/fips/205/final
