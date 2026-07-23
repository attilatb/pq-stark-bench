# Methodology

This document defines exactly what PQ-STARK-BENCH measures, how it measures it, and what it refuses to claim. It is written to be adversarial against itself: every number we publish should survive a hostile reviewer who has read the same primary sources we did.

All source access dates in this document are 2026-07-23 unless stated otherwise.

## 1. Scope: what we measure

The headline metric is **amortized prover wall-clock seconds per signature, as a function of batch size N, per (scheme, parameter set, prover, hardware) cell.**

For every cell we record and publish:

| Metric | Unit | Why |
| --- | --- | --- |
| Prover wall-clock, total | nanoseconds | The only metric that is comparable across proving systems |
| Prover wall-clock per signature | nanoseconds | Total divided by N. Derived by us, always labeled as derived |
| Peak resident set size (RSS) of the prover process | bytes | The binding constraint on batch size in practice |
| Mean and peak CPU utilization | percent, summed across logical cores | A 16-core machine fully loaded reports 1600, not 100 |
| Proof size | bytes | The only quantity that actually amortizes with N |
| Guest execution cost | prover-native units (see 3.1) | Reported per prover, never on a shared axis |
| Native (non-zkVM) verify time for the same scheme | nanoseconds | The denominator for the "zkVM overhead factor" |

We measure **signature verification only**, inside the guest. Key generation and signing happen on the host, outside the timed region, and are recorded separately as native baselines.

## 2. Scope: what we deliberately do not measure

These are exclusions by design, not gaps we hope nobody notices.

- **We do not treat "does amortized prover cost fall with batch size" as an open question.** It does not. Prover time and prover RAM grow approximately linearly in N; only proof bytes amortize. This was published for a hash-based PQ signature in a STARK by IACR ePrint 2021/1048 (AsiaCCS 2022, Table 3: n = 128 to 1024, 2.5 s to 19.7 s prover time and 0.9 GB to 7.4 GB prover RAM at 96-bit STARK security, on "an 8-core Intel Core i9 processor @ 2.4 GHz with 32 GB of RAM"), it is restated in the facebook/winterfell README ("Trace time and prover RAM ... grow pretty much linearly with the size of the computation ... Proof size and verifier time grow much slower than linearly (actually logarithmically)"), it is independently visible in arXiv:2603.07974 Table 7 (prove/tx 14.29, 14.45, 14.46, 15.76, 14.77 ms at N = 1, 2, 4, 8, 16 while proof/tx falls 122 KB to 13 KB), and it is visible again in the leanEthereum/leanBench committed result files. We measure the curve because the *constants* per FIPS scheme are unpublished, not because the *shape* is unknown. Any framing of the shape as a discovery is wrong and we will not use it.
- **We do not claim any "first."** PQ signature verification inside a STARK dates to at least 2021/1048. Falcon in Cairo on Starknet was demonstrated by BTQ with StarkWare in a post published **Sep 18, 2023** (not 2024), and that post publishes no proving time, no step count, no constraint count, no RAM, and no proof size. Hash-based PQ aggregation inside RISC Zero was published as HAPPIER (LightSec 2025, LNCS 16216, first online 31 Jan 2026). The narrow cell we fill is stated in section 11.
- **We do not measure on-chain verification gas.** Cairo steps and Starknet L2 gas are a different axis. OpenZeppelin/cairo-pq-verifiers publishes those (Falcon-512 bare verify: 97,681 steps / 12,045,449 L2 gas for the non-standard Poseidon variant, up to 413,819 steps / 60,359,788 L2 gas for the standard SHAKE-256 direct variant) and publishes **no proving time, prover, hardware, RAM, or proof size**. Placing a step count next to a prover-second on the same chart is a category error and we do not do it.
- **We do not measure signing or key generation inside a proof.** Falcon signing in particular uses floating point in reference implementations; guest targets here are integer-only soft-float and the result would measure the emulation library, not the scheme.
- **We do not publish a vendor cost-per-proof.** See section 8.

## 3. The cross-prover comparability problem

### 3.1 Why RISC Zero cycles and SP1 cycles are not the same unit

Three independent reasons, each sufficient on its own.

**Different instruction set.** RISC Zero's guest target is `riscv32im-risc0-zkvm-elf` and its specification states the zkVM implements "the `RV32IM` instruction set." SP1 v6.3.1's build crate declares `pub const DEFAULT_TARGET: &str = "riscv64im-succinct-zkvm-elf";` and its `zkevm/README.md` states "Target triple: `riscv64im-succinct-zkvm-elf` (RV64IM, LP64, soft-float)." The same Rust source therefore compiles to different instruction streams with different limb widths. For lattice arithmetic this is not a rounding difference: a 64-bit multiply changes the entire cost structure of NTT and modular reduction. Note that SP1's own "Compiling" documentation page still says `riscv32im-succinct-zkvm-elf`, which contradicts both its install page and its v6.3.1 source. That page is stale. Anyone citing it for "SP1 is RV32" is citing a documentation bug.

**Different accounting for the same work.** RISC Zero charges memory paging explicitly: "A page-in or page-out operation takes between 1094 and 5130 cycles; 1130 cycles on average," and charges 2 cycles for AND/OR/XOR, div, rem, and shift-right against 1 for add and load. SP1 v6 charges a flat clock increment per syscall regardless of which precompile is invoked; its `syscall_code.rs` states "each syscall instruction increments the clock by 256 additionally." SP1 has no paging-cycle analogue at all.

**The vendor disowns the unit.** Succinct introduced prover gas (PGUs) precisely because cycles mispredict proving cost, documenting that "Two programs of the same cycle counts may require significantly different proving times" and giving a worked example where a keccak256 workload at approximately 5.6 million cycles proves faster than an ECDSA recovery workload at approximately 4.4 million cycles. RISC Zero's `SessionInfo::cycles()` is documented as "the total number of user cycles across all segments, **without any overhead for continuations or po2 padding**," so it is a lower bound on proven work, not a measure of it.

### 3.2 Our primary metric instead

**Prover wall-clock seconds and peak RSS, on one pinned machine, for an identical workload definition, with proof bytes co-reported.** That is the only quantity whose semantics do not change between proving systems.

Guest execution counts are still recorded, because they are cheap, deterministic, and useful for attribution within a prover. They appear in results files and in per-prover panels. They never share an axis across provers. Specifically:

- RISC Zero: `SessionInfo::cycles()` **and** the padded per-segment po2 totals, both stored. Publishing only the former understates work.
- SP1: `report.total_instruction_count()` **and** `report.gas` (prover gas). Note the SP1 `ExecutionReport` serde format is documented as stable only within a single SP1 version, so cycle and PGU figures are comparable only within a pinned version.

### 3.3 Security level: we cannot normalize it, so we disclose it

The two systems do not default to the same soundness level, and one of them does not publish a number at all for its current prover. RISC Zero documents 96 bits for the RISC-V prover and 99 for recursion. SP1 documents 100 bits conjectured for Turbo and publishes **no soundness-bit figure for Hypercube**.

We therefore do not attempt a soundness-normalized time. Any attempt would require re-parameterizing FRI inside at least one prover, which changes the artifact under test into something the vendor does not ship, and would make our numbers non-reproducible against a released binary.

**Mandatory caveat, which appears verbatim as a footnote on every cross-prover chart we publish:**

> Cross-prover wall-clock comparison. These provers are not configured to the same soundness level. RISC Zero documents 96 bits (RISC-V prover) and 99 bits (recursion); SP1 documents 100 bits conjectured for Turbo and publishes no soundness figure for Hypercube. Lower claimed soundness generally means less prover work. This chart compares shipped default configurations, not equal-security configurations. It is a deployment comparison, not a cryptographic one.

A second mandatory caveat covers precompile asymmetry:

> Precompile asymmetry. Neither prover ships a lattice, NTT, ML-DSA, or Falcon accelerator. Both ship elliptic-curve accelerators used by the classical baselines (SP1: `ED_ADD`, `ED_DECOMPRESS`, `SECP256K1_*`, `SECP256R1_*`; RISC Zero: patched `curve25519-dalek`, `k256`, `p256`). Both can accelerate the Keccak-f[1600] permutation underlying SHAKE (SP1 via the patched `sha3`/`tiny-keccak` crates routing to `KECCAK_PERMUTE`; RISC Zero via patched `tiny-keccak`, which requires the `unstable` feature on both `risc0-zkvm` and `risc0-build`). Neither has a SHAKE-specific circuit. A PQ-versus-classical ratio drawn from this chart therefore overstates the intrinsic PQ penalty and understates the value of a missing lattice precompile.

Note that the round-1 statement "RISC Zero does not list a sha3/SHAKE precompile" is half wrong and is corrected above: no `sha3` crate appears in RISC Zero's patch table, but `tiny-keccak` 2.0.2 is patched, and the v3.0.1 release notes state "Stabilize bigint and keccak features." Every ML-DSA-44 cell on RISC Zero is therefore run **twice**, once with the stock `sha3` crate and once with patched `tiny-keccak`, and both numbers are published. The delta is a result, not an implementation detail.

### 3.4 Third-lane note (Cairo / Stwo)

If and when a Cairo/Stwo lane is added, it is a third incomparable unit again (Cairo steps, plus a prover whose parameters are set by a `prover_params.json`, e.g. `pow_bits` 26, `log_blowup_factor` 1, `n_queries` 70 in starkware-bitcoin/s2morrow). It gets its own panel under the same wall-clock headline, with its FRI parameters printed next to the chart. Note also that s2morrow's `make *-prove` targets depend on `ssh://git@github.com/m-kus/proving-utils.git`, which is not publicly resolvable, so that lane is not third-party reproducible as published.

## 4. Statistical protocol

- **Iterations.** n = 10 timed iterations per (scheme, param set, prover, batch size, machine) cell for cells under 60 seconds per iteration. For cells over 60 seconds per iteration, n = 5, and the reduced n is recorded in the results file as `timing.n` so it is visible without reading this document. We do not silently vary n.
- **Warmup.** Two discarded warmup iterations precede every timed set. This is not cosmetic. RISC Zero documents that "the GPU (e.g. Metal or CUDA) kernels may need to be JIT compiled. This can take a few minutes, but should only happen once," so a first-run measurement records a one-off compilation as prover time. Prover client construction is also expensive in SP1 and is hoisted out of the timed region and reused (`Arc`-shared) across the sweep.
- **What the timer covers.** Proof generation only: from the call that begins proving to the call's return. Guest ELF build, prover key setup, witness/input serialization, and receipt serialization are outside the timed region and are recorded as separate fields. Any setup that a proving library performs lazily on first call (DFT twiddle tables, bytecode init) is absorbed by warmup, and we state that explicitly rather than pretending the timer is pure.
- **Reported statistics.** We publish **median (p50)** as the headline, plus **p95**, plus min, max, mean, and standard deviation. Median is the headline because it is robust to the OS scheduling and thermal excursions described in section 5; p95 is published alongside because a benchmark that hides its tail is a benchmark that hides its worst behavior. Mean is published for continuity with prior work that reports means, not because we prefer it.
- **Raw samples.** Every individual sample is stored in the results file as a `samples_ns` array. We do not publish only derived statistics. This is a deliberate borrowing from leanBench's schema and it makes our numbers re-analyzable by someone who disagrees with our choice of statistic.
- **Outlier policy.** We do not delete outliers. Nothing is trimmed, winsorized, or dropped. All samples are retained and published, and the median absorbs them. If a run contains a sample more than 3x the median we do not remove it; we flag the run with `anomaly: true`, keep the data, and note the suspected cause (thermal, background process, page cache) in the run `notes` field. A benchmark with a discard rule is a benchmark with a tuning knob.
- **Seeding.** All randomness (key generation entropy, message generation, per-iteration key variation) is drawn from a fixed seed, recorded in the results file. Fixtures are generated once and reused byte-identically across provers so that both provers verify the same signatures over the same messages. Cross-prover comparison over different fixtures is not a comparison.
- **Correctness gate.** Every timed run is preceded by an untimed correctness check: the guest must accept a valid signature and reject a tampered one (single-bit flip in the signature, and separately in the message). A cell that does not pass both is not published as a timing. This gate exists because a stub that returns `true` for any input of the right shape produces excellent benchmark numbers, and at least one published PQ verifier crate in this space is exactly that.

## 5. The laptop problem

The primary development machine is an Apple M3 Max (aarch64 macOS, 16 cores, 64 GB unified memory, no CUDA). It is a laptop. Laptops throttle.

Facts that constrain what we can honestly claim there:

- macOS does not expose a supported CPU frequency governor or turbo lock. We cannot pin clocks the way a Linux benchmark can. There is no `performance` governor to set and no documented way to disable P-core boost.
- Sustained multi-core proving heats the package. A 20-minute sweep is not measuring the same silicon speed at minute 19 that it measured at minute 1.
- Memory is unified. Peak RSS competes with the GPU and with the OS compositor in a way it does not on a discrete-GPU Linux box.
- The two provers do not get the same backend on this machine. RISC Zero v3.0.6 uses Metal on Apple Silicon automatically. SP1 v6.3.1 has no GPU path here at all: CUDA is documented as "only supported on Linux x86_64" and AVX acceleration is Intel-only, so SP1 runs scalar CPU. A head-to-head wall-clock chart produced on this Mac compares a GPU-accelerated prover against a scalar-CPU prover and measures backend availability, not prover design.
- Groth16 wrapping is not available symmetrically. RISC Zero documents that "The Groth16 prover currently _only_ works on x86 architecture, and so Apple Silicon is _currently unsupported_ (even via Docker)." SP1 documents Groth16 and PLONK wrapping via Docker with "roughly 14GB for Groth16 and 60GB for PLONK" for the wrap step; on 64 GB shared with macOS, PLONK is very likely infeasible and Groth16 is tight.

What we do about it:

1. **Machine classes are first-class and never mixed on one axis.** Every results file carries `machine.class`, one of `laptop-thermal-unpinned`, `workstation-pinned`, or `ci-shared`. Charts render a different series style per class and the legend names the class. There is no chart in this benchmark that averages across classes.
2. **The Mac is a development and soak machine.** Headline cross-prover figures are produced on a pinned Linux x86_64 host where both provers have comparable backend availability. The Mac lane is published in full, labeled `laptop-thermal-unpinned`, and is presented as an "on the hardware a developer actually owns" datapoint, not as the cross-prover result.
3. **Cooldown and interleaving.** A 90-second idle cooldown separates cells. Batch sizes are run in interleaved order rather than monotonically ascending, so that a monotone thermal drift cannot masquerade as a monotone scaling result. This is the single most likely way a laptop benchmark manufactures a fake trend, and interleaving is the cheapest defense.
4. **Thermal telemetry is sampled** at 100 ms alongside CPU and RSS, and the sampled series is stored. If a reviewer wants to argue a run was throttled, the data to make that argument is in our file.
5. **CPU-only control runs.** For the Mac lane we additionally run RISC Zero with GPU acceleration disabled via its documented CPU/GPU feature flags, so a genuine CPU-versus-CPU pair exists alongside the Metal best-case. Both are published.
6. **CI results are labeled separately and are never headline numbers.** CI runners are shared, virtualized, of unknown co-tenancy, and of unstable instance type. CI exists to detect *regressions in our own harness* (does it still build, does the correctness gate still pass, did a number move by more than a threshold), not to produce publishable performance figures. Every CI-produced file carries `machine.class: ci-shared` and `publishable: false`. The website filters them out of all performance charts by default.

## 6. Required metadata in every results file

One JSON file per run, named `results/<ISO8601-timestamp>__<machine-fingerprint>.json`. Files are append-only and are never hand-edited. The fingerprint is deliberately coarse so that an OS point release does not fragment a machine's history.

Required top-level keys:

```
run_id                ISO8601 timestamp + machine fingerprint
timestamp             ISO8601 with timezone
publishable           bool (false for CI and for any run failing the correctness gate)
machine {
  fingerprint         stable coarse hash
  class               laptop-thermal-unpinned | workstation-pinned | ci-shared
  cpu_model           verbatim vendor string
  cpu_arch            x86_64 | aarch64
  physical_cores, logical_cores
  memory_gb
  gpu                 model string or null
  os, kernel
  label               human name, e.g. "M3 Max 16c/64GB" or "c4-standard-16"
  ac_power            bool (laptops only)
}
toolchain {
  rustc               full version string
  prover_version      exact pinned version, e.g. "risc0 3.0.6" / "sp1 6.3.1"
  prover_version_cmd  the command output we captured, verbatim
  guest_target        e.g. riscv32im-risc0-zkvm-elf | riscv64im-succinct-zkvm-elf
  build_profile       always "release"
  git_sha             this repository, dirty flag included
}
prover {
  name                risc0 | sp1 | stwo
  backend             cpu | metal | cuda | avx2 | avx512
  proof_mode          core | composite | succinct | compressed | groth16 | plonk
  soundness_bits      integer, or null with a "not published" note
  fri_params          object, or null
  segment_limit_po2   RISC Zero: read at runtime, never assumed
  precompiles_used    array of patched crates and syscalls actually exercised
}
scheme {
  name                ML-DSA | SLH-DSA | FN-DSA/Falcon | Ed25519 | ECDSA-secp256k1 | ...
  param_set           e.g. ML-DSA-44, SLH-DSA-SHAKE-128s, Falcon-512
  spec_ref            e.g. "FIPS 204" / "FIPS 205" / "FIPS 206 (draft)"
  conformant          true | false
  nonconformance      null, or a verbatim description (e.g. "hash-to-point XOF
                      replaced with Poseidon; not interoperable with a compliant signer")
  implementation      crate name + version + git sha
}
workload {
  batch_size          N
  topology            flat | tree(fan_in)
  fixture_seed
  fixture_sha256      hash of the exact signature set verified
}
timing {
  unit                "ns"
  n                   iteration count actually used
  warmup_discarded    integer
  samples_ns[]        every raw sample, undeleted
  min, p50, p95, max, mean, stddev
}
resources {
  rss_bytes { mean, peak }
  cpu_percent { mean, peak }     summed across logical cores
  thermal[]                      optional sampled series
  n_samples, interval_ms
}
proof {
  bytes
  verify_ns_p50
}
cost { see section 8 }
anomaly               bool
notes                 free text, including any known contamination
```

Absent data is written as an explicit `null` with a sibling `*_reason` string such as `"not published by vendor"` or `"not measurable on this platform"`. We never omit a key to hide a gap, and we never write a zero where we mean unknown.

## 7. Conformance is a first-class axis, not a footnote

Cheap PQ numbers are frequently cheap because the scheme was modified. The clearest published example is OpenZeppelin's Cairo verifiers, where three of five Falcon-512 variants swap the standard SHAKE-256 hash-to-point for BLAKE2s or Poseidon and are self-labeled non-standard, and the repository itself quantifies the penalty: SHAKE-256 versus native-Poseidon is "4.16x the gas and 3.20x the steps." The same distortion appears in feltroidprime/s2morrow, which swaps SHAKE-256 for Poseidon.

We do not treat this as a scandal. OpenZeppelin discloses it clearly and ships conformant variants. We treat it as an axis:

- Every cell carries `scheme.conformant`. A cell is conformant only if it would interoperate with a compliant signer (for Falcon-512, `falcon.py`; for ML-DSA, the FIPS 204 vectors).
- Non-conformant cells are rendered in a visually distinct series and are never used in a headline number.
- Where both exist, we publish the conformance penalty explicitly as its own result.
- We never write "FIPS-conformant Falcon" unqualified. FIPS 206 is draft. The correct phrasing is "conformant to the Falcon specification / interoperable with a compliant signer."

## 8. The dollar-cost model

There is no vendor price to cite, and anyone who quotes one is quoting a dead or auction-determined number.

- **RISC Zero Bonsai does not exist.** `dev.risczero.com/api/bonsai/bonsai-overview` returns HTTP 404, and the Boundless documentation states "Bonsai was RISC Zero's centralized proving service, delivering proofs via an API. As of December 2025, Bonsai is no longer available." Its replacement, Boundless, is a bid-based marketplace with no published fixed rate. Any benchmark citing Bonsai dollars-per-proof is citing a discontinued product.
- **SP1's network is auction-priced.** Succinct's FAQ states "The price per PGU is set through a competitive auction," with "$PROVE Cost Estimate = Base Fee + PGUs * Price per PGU," and that "Both are dynamic." There is no published PGU price and no published base fee.

Therefore our dollar figure is a **rental-equivalent cost, computed by us, from a named on-demand cloud instance price**, and it is always labeled as our derivation, never as a market price.

The formula, stated in full on the cost page and reproduced in every results file:

```
usd_per_batch = (prover_wall_clock_p50_seconds / 3600) * instance_usd_per_hour
usd_per_signature = usd_per_batch / N
```

Rules that make the number honest:

1. **One reference instance, named, with region.** Dollar figures are computed only for runs on the pinned Linux x86_64 workstation-class lane whose CPU, core count, and memory match a specific publicly listed cloud instance type. The instance type, region, on-demand hourly USD price, the exact vendor pricing-page URL, and the date the price was read are all stored in the `cost` block of the results file and printed under every cost chart. Prices are re-read and re-stamped at each release; historical files keep their original price and date.
2. **On-demand list price only.** No spot, no committed-use, no reserved, no negotiated discount, no free tier. These would make the number unreproducible by a reader.
3. **Compute time only.** We charge wall-clock proving seconds at the instance rate. We do not include storage, egress, orchestration, engineering time, or amortized setup. The model therefore **understates** true operational cost, and we say so in that direction explicitly. An understating model is the safe direction for a benchmark whose thesis is that PQ proving is expensive: it cannot be accused of inflating the penalty.
4. **No dollars for the laptop lane.** An M3 Max is not a rentable instance. Assigning it a cloud price would be fabrication. Laptop-class runs carry `cost: null` with `cost_reason: "no equivalent rentable instance"`.
5. **No dollars for GPU lanes without a matching instance.** If a GPU configuration does not correspond to a listed instance type, it gets no dollar figure.
6. **PGUs are published raw.** For any SP1 network runs we publish the PGU count (deterministic, from `report.gas`) and, if a network proof was actually purchased, the observed clearing price with its timestamp. We never extrapolate a clearing price to a different date.

Standing caveat printed beneath every cost chart:

> Cost is derived by us, not quoted by a vendor. It is prover wall-clock seconds on a named on-demand cloud instance at that instance's list price on the stated date, and nothing else. It excludes storage, egress, orchestration, and engineering time, and therefore understates real operational cost. It is not a market price for a proof: RISC Zero's Bonsai service was discontinued in December 2025, and Succinct's network prices proofs by competitive auction with no published rate.

## 9. Reproduction instructions

Every published figure must be reproducible by a third party from a clean machine. The `REPRODUCE.md` shape is:

1. **Provenance header.** Repository URL, exact commit SHA, license, and the results file(s) that a given chart was rendered from.
2. **Toolchain pinning, exact.** The install command and the pinned version for each prover, plus the command whose output we captured to prove the version. Nothing resolves to "latest." Concretely: `rzup install cargo-risczero <version>` and `rzup install r0vm <version>` with matching versions, verified via `cargo risczero --version`; `curl -L https://sp1up.succinct.xyz | bash` then `sp1up` at a pinned release, verified via `cargo prove --version`. Rust toolchain version pinned for both host and guest. Note that `docs.rs` and GitHub release tags can disagree transiently, so we record the tool's self-reported version string, not a registry lookup.
3. **Fixture generation.** A single command that regenerates the exact signature fixtures from the recorded seed, plus the expected SHA-256 of the fixture set. If the hash does not match, stop.
4. **A single sweep command per lane** with the machine class as an explicit argument, so a reader cannot accidentally produce a laptop file labeled as a workstation file.
5. **Expected runtime and expected peak RSS per lane**, so a reader knows before starting whether their machine can complete it.
6. **A "your numbers will differ" section** stating which differences are expected (absolute wall-clock, absolutely; ratios between schemes on the same machine, much less) and which differences indicate a real problem (correctness gate failure, proof size mismatch, guest cycle count mismatch at a pinned prover version).
7. **Known-blocked steps, named.** Groth16 wrapping cannot be reproduced on Apple Silicon for RISC Zero; PLONK wrapping is likely infeasible on a 64 GB machine for SP1. These are stated up front, not discovered by the reader at hour three.
8. **Contribution path.** New machines are welcome; a run lands as `results/<timestamp>__<fingerprint>.json` and is committed unedited. Contributed files must include the full metadata block in section 6 or they are not merged.

## 10. Anti-patterns we commit to avoiding

These are failure modes we observed in the literature while building this benchmark, recorded so that we are held to the same standard.

- **Extrapolation presented as measurement.** HAPPIER's published per-batch totals are computed as `mean(leaf_proof) + merge_node * log2(N)/log2(arity)` from runs whose logs show `Bitfield: 0b00000011` at every N, meaning two distinct signatures were ever aggregated. We publish only measured wall-clock for the batch actually proven. If we ever publish a modeled projection it is in a separate column labeled `modeled`, with the formula printed.
- **Missing hardware.** A 22.07 s proving time on an outsourced network with undisclosed, time-varying hardware is not a machine-pinned measurement. Every timing we publish names its machine.
- **Numbers that do not self-check.** A published cycle breakdown summing to 1,100,000 against a stated total of 5,625,411 is a five-fold internal contradiction visible on the same page. Our results files are validated by a script that checks internal arithmetic (samples versus statistics, per-node versus totals, percentages versus absolutes) before a file is committed.
- **Dead artifact links.** IACR ePrint 2021/1048's artifact reference is `github.com/anonauthorsub/asiaccs_2021_440`, which 404s; the facebook/winterfell attribution is an inference, not a statement by the paper. One widely circulated ML-DSA-in-SP1 result links a GitHub repository that has never received a commit. We check every link we publish on every release and record the check date.
- **Trusting page summarizers over raw files.** An automated summary of starkware-bitcoin/s2morrow reported "Stwo proving benchmarks" as completed when the raw README says `- [ ] Stwo proving benchmarks`. A summary of OpenZeppelin's benchmark table silently dropped both `direct` variants, including the highest-cost row. All figures in this benchmark are extracted from raw files or APIs, never from a rendered-page summary.
- **Blending two published tables of nominally the same thing.** The 2021/1048 paper and the Winterfell README disagree at n = 1024, 123-bit (25.7 s / 9.5 GB / 165 KB versus 20.5 s / 7.6 GB / 152 KB). We cite them separately and never merge them into one series.

## 11. Known limitations

Written so that a hostile reviewer finds nothing here they could have used against us.

1. **Our novelty claim is narrow and we state it narrowly.** We do not claim the first PQ signature in a STARK, the first PQ signature in a zkVM, the first batch sweep, or the first amortization curve. All four exist in prior work. What appears to be unpublished as of 2026-07-23 is **measured prover wall-clock and peak RAM versus batch size for a FIPS-standardized signature scheme (ML-DSA-44 in particular) in a general-purpose zkVM, on named hardware, across more than one prover, with conformant and non-conformant variants separated.** If that turns out to exist, this benchmark is a replication, which is still worth publishing, and we will relabel it as such.
2. **Cross-prover soundness is not normalized and cannot be, at shipped defaults.** See 3.3. Every cross-prover chart carries the caveat. A reviewer who says "these are not equal-security comparisons" is correct, and we said it first.
3. **SP1's Hypercube soundness is not published at all.** We report `soundness_bits: null` with a reason string rather than guessing.
4. **The precompile landscape is asymmetric and favors the classical baselines.** See 3.3. The PQ penalty we measure is a penalty against *hand-accelerated* classical schemes and *unaccelerated* lattice arithmetic. It is a deployment-reality number, not a statement about the schemes.
5. **The laptop lane cannot pin clocks.** macOS exposes no supported governor or turbo lock. We mitigate with warmup, cooldown, interleaving, medians, thermal telemetry, and class labeling, and we do not headline the laptop lane. We do not claim the mitigations eliminate the problem.
6. **Backend availability differs by platform.** RISC Zero gets Metal on Apple Silicon; SP1 does not get any GPU path there. The Mac lane's cross-prover numbers are backend comparisons, and are labeled as such rather than suppressed.
7. **Groth16 and PLONK coverage is incomplete.** RISC Zero's Groth16 prover is x86-only and documented as unsupported on Apple Silicon even via Docker. SP1's PLONK wrap is documented at roughly 60 GB for the wrap step and is likely infeasible on 64 GB shared with an OS. Where a wrapping mode is unavailable we publish `null` with a reason, not a substitute.
8. **RISC Zero's default `segment_limit_po2` is not published** (only bounded to the interval [13, 24] by `MIN_CYCLES_PO2` and `MAX_CYCLES_PO2`), and peak prover RAM is roughly linear in segment size. We read the value at runtime and record it rather than assuming a default, but this means our RAM figures are only reproducible against the recorded value, not against "the defaults."
9. **Guest execution counts are version-locked.** SP1's `ExecutionReport` serde format is documented as stable only within a single SP1 version. Cycle and PGU figures are comparable within a pinned version and not across versions.
10. **We have no citable native aarch64 baseline for any of our schemes.** The PQShield nist-sigs-zoo is x86-only, has no keygen column for signatures, has no secp256k1, mixes provenance across schemes (`perf_source` is "OQS bench" for ML-DSA and Falcon, "submission document" for SLH-DSA, "OpenSSL bench" for EdDSA, absent for ECDSA), and its own ML-DSA-44 sign figure (172,926 cycles) disagrees roughly 1.9x with pq-crystals' official Skylake AVX2 figure (333,013 cycles). eBACS has aarch64 but only Raspberry Pi class, and its `dilithium2`/`falcon512` entries are round-3 submissions, not the final FIPS parameterizations. So our native aarch64 baselines are self-measured and unvalidated against any external source. We say so rather than implying external corroboration exists.
11. **The dollar figure is a model, not a price, and it understates.** See section 8.
12. **N is bounded by memory, not by patience.** Since prover RAM grows approximately linearly in N, the largest batch we report per cell is set by the machine, and different machines will therefore have different maximum N. Curves are not truncated silently; the maximum attempted N and the failure mode (OOM, guest memory limit, session limit) are recorded.
13. **Single-implementation risk.** Each scheme is measured through one guest implementation. A faster implementation would move every number for that scheme. Our figures characterize the implementation named in `scheme.implementation`, on that prover, not the scheme in the abstract.
14. **Falcon signing is out of scope and Falcon verification alone is measured.** Reference Falcon signing uses floating point; both guest targets are integer-only soft-float. Any signing figure would measure emulation.
15. **We are not affiliated with any prover vendor.** RISC Zero is Apache-2.0 and SP1 is MIT (LICENSE-MIT, "Copyright (c) 2023 Succinct Labs"; we did not verify a companion Apache file, so we describe SP1 as MIT rather than dual-licensed). Neither vendor has reviewed, funded, or endorsed these results, and nothing here grants us use of their trademarks beyond descriptive reference.