## TRACK A TOOLING DECISION — RISC Zero vs SP1 on M3 Max (aarch64 macOS, no CUDA)

### 1. RECOMMENDATION
**Start with SP1, then add RISC Zero second** — SP1's patched-crate set is the only one of the two that covers **sha3/SHAKE** (`sha3` 0.10.8 **and** 0.11.0), which every PQ scheme in scope (Falcon hash-to-point, ML-DSA, XMSS/SPHINCS+) is bottlenecked on, and its `ExecutionReport` is `Serialize`/`Deserialize` (added v6.2.3) so cycle data drops straight into the results JSON.

### 2. CAN EACH RUN LOCALLY ON macOS aarch64?

| | (a) execute-only cycle counting | (b) real proof generation |
|---|---|---|
| **SP1** | Yes | Yes, **CPU-only scalar** |
| **RISC Zero** | Yes | Yes, **Metal GPU accelerated** |

- **SP1 runs on macOS**: *"SP1 can currently be run natively on Linux and macOS"*; prebuilt binaries *"built on Ubuntu 20.04 (22.04 on ARM) and macOS"*, with "ARM Mac" listed for `sp1up`. ([docs.succinct.xyz install](https://docs.succinct.xyz/docs/sp1/getting-started/install), accessed 2026-07-23)
- **SP1 has no GPU on Apple Silicon.** CUDA is *"Linux x86_64"* only, requiring *"CUDA 12 runtime"*, *"Compute Capability 8.0 or higher"*, *"24GB or more VRAM"*; CPU acceleration is *"AVX256 and AVX512 acceleration on Intel x86 CPUs"*. **No mention of Apple Silicon or Metal anywhere on that page.** ([hardware-acceleration](https://docs.succinct.xyz/docs/sp1/generating-proofs/hardware-acceleration), accessed 2026-07-23)
- **Evidence SP1 core+recursion proving actually works on M-series**: issue [succinctlabs/sp1#2289](https://github.com/succinctlabs/sp1/issues/2289) (OS field: "macOS (Apple Silicon)", opened 2025-05-12, closed 2025-06-11) — a spurious warning, but *"the proof was successfully generated and verified later"*. SP1's Groth16 wrap on macOS goes through Docker (`gnark-ffi/src/ffi/docker.rs`, per [#2287](https://github.com/succinctlabs/sp1/issues/2287)); an older native gnark-ffi link failure on darwin/arm64 is [#841](https://github.com/succinctlabs/sp1/issues/841) (closed 2024-05-30). All accessed 2026-07-23.
- **RISC Zero explicitly supports arm64 macOS** as a first-class install target (x86-64 Linux and arm64 macOS; other combos need manual build). ([dev.risczero.com/api/zkvm/install](https://dev.risczero.com/api/zkvm/install), accessed 2026-07-23)
- **RISC Zero has Metal**: *"On MacOS, when using a machine with Apple Silicon (such as the M-series MacBooks), RISC Zero will use the integrated Metal compute cores."* But: *"The Groth16 prover currently only works on x86 architecture, and so Apple Silicon is currently unsupported (even via Docker)."* ([local-proving](https://dev.risczero.com/api/generating-proofs/local-proving), accessed 2026-07-23) — irrelevant for a cycles/STARK benchmark, blocking only if you want on-chain wrapping.

### 3. INSTALL + CURRENT VERSION (SP1)
```bash
curl -L https://sp1up.succinct.xyz | bash
sp1up
cargo prove --version
```
Deps listed: Git, Rust, Docker, `protoc`. ([install docs](https://docs.succinct.xyz/docs/sp1/getting-started/install), accessed 2026-07-23)

**Current version: v6.3.1, published 2026-06-25** (latest stable; v6.3.0 2026-06-20, v6.2.4 2026-06-08). Source: GitHub releases API for `succinctlabs/sp1`, accessed 2026-07-23.

**RISC Zero for comparison:** `curl -L https://risczero.com/install | bash && rzup install`. Latest stable **v3.0.6, published 2026-07-17**; note a **v5.0.0-rc.1 prerelease (2026-01-15)** exists ahead of the 3.0.x stable line — version numbering is non-linear here, pin explicitly. Source: GitHub releases API for `risc0/risc0`, accessed 2026-07-23.

### 4. CYCLE COUNTS WITHOUT PAYING FOR PROVING

**SP1** — `execute()` never invokes the prover:
```rust
let report = client.execute(ELF, &stdin).run().unwrap();
let total_compute_cycles = report.cycle_tracker.get("compute").unwrap();
```
Guest-side annotations: `println!("cycle-tracker-start: label")` / `cycle-tracker-end`, or `cycle-tracker-report-start`/`-end` to **accumulate across multiple invocations** (exactly what you want for batch-size sweeps), or `#[sp1_derive::cycle_tracker]`. Caveat, verbatim: *"The `cycle_tracker` and `invocation_tracker` fields in the `ExecutionReport` are only populated when the `profiling` feature is enabled for `sp1-sdk`"* — so `sp1-sdk = { version = "...", features = ["profiling"] }`. ([cycle-tracking](https://docs.succinct.xyz/docs/sp1/optimizing-programs/cycle-tracking), accessed 2026-07-23)

**RISC Zero** — dev mode plus pprof:
```bash
RISC0_PPROF_OUT=./profile.pb RUST_LOG=info RISC0_DEV_MODE=1 RISC0_INFO=1 cargo run
go tool pprof -http=127.0.0.1:8000 profile.pb
```
Docs: *"We recommend running profiling in dev mode to avoid unnecessary proving time."* ([profiling](https://dev.risczero.com/api/zkvm/profiling), accessed 2026-07-23)

### 5. PRECOMPILES THAT MATTER FOR OUR SCHEMES

| Primitive | SP1 v6.0.0 patch tags | RISC Zero |
|---|---|---|
| **sha2** | `sha2` 0.9.9, 0.10.6, 0.10.8, 0.10.9 (`patch-sha2-0.10.9-sp1-6.0.0`) | `sha2` 0.9.9, 0.10.6–0.10.8 |
| **keccak** | `tiny-keccak` 2.0.2 (`patch-2.0.2-sp1-6.0.0`) | `tiny-keccak` 2.0.2 |
| **sha3 / SHAKE** | **`sha3` 0.10.8, 0.11.0** (`patch-sha3-0.11.0-sp1-6.0.0`) | **not listed** |
| **bigint** | `crypto-bigint` 0.5.5 | `crypto-bigint` 0.5.2–0.5.5; *"Stabilize bigint and keccak features"* (v3.0.1 notes) |
| **ed25519** | `curve25519-dalek` 4.1.3, `curve25519-dalek-ng` 4.1.1 | `curve25519-dalek` 4.1.0–4.1.3 |
| classical baselines | `k256`, `p256`, `secp256k1`, `bls12_381` 0.8.0, `rsa` 0.9.6, `substrate-bn` | `k256`, `p256`, `blst`, `bls12_381` 0.8.0, `rsa` 0.9.6, `c-kzg` |

Sources: [SP1 precompiles](https://docs.succinct.xyz/docs/sp1/optimizing-programs/precompiles) and [RISC Zero precompiles](https://dev.risczero.com/api/zkvm/precompiles), both accessed 2026-07-23.

**Neither has any lattice/NTT/ML-DSA/Falcon precompile.** Consequence for the benchmark, and it must be disclosed in every chart: your PQ numbers are **unaccelerated RISC-V**, while the Ed25519/ECDSA control line is **precompile-accelerated**. Reporting a raw "PQ penalty multiplier" without that caveat overstates it. Report both accelerated and `--no-default-features` unaccelerated baselines.

### 6. BIGGEST RISK + FALLBACK

**Risk: SP1 has zero hardware acceleration on this machine.** No CUDA (Linux x86_64 only), no Metal, and AVX is Intel-x86-only — so every real SP1 proof on the M3 Max is scalar CPU. Multi-million-cycle PQ workloads will be slow and RAM-hungry: the closest public data point is [ebulgin/qrypta-pu](https://github.com/ebulgin/qrypta-pu) (accessed 2026-07-23) claiming *"~40 min, needs ~14 GB RAM"* for a single ML-DSA-44 Groth16 on unnamed CPU — and that repo's own test comment says "~10 min", so the figure is internally inconsistent and **unverified**. On 64 GB you may fit one large proof but not a comfortable batch sweep.

**Fallback, in order:**
1. **Split the axes.** Cycle counts and constraint-shaped metrics from SP1 `execute()` locally on the M3 Max (cheap, deterministic, machine-independent, hardware-normalizable) — publish those first. Treat wall-clock proving as a separate, clearly hardware-labelled axis.
2. **Use RISC Zero as the local wall-clock prover**, since Metal actually engages on M-series. This also satisfies your standing cross-prover requirement (task #2) rather than being a detour.
3. **If neither gives usable wall-clock at batch size**, move proving to a Linux x86_64 + CUDA box and keep the M3 Max for execution/cycles. Never mix machines within one chart.

**Explicitly unverified:** SP1 proof-type page gives no platform requirements at all for Groth16/PLONK (*"does not specify Docker requirements, x86 requirements, or mention any ARM, Apple Silicon, or macOS limitations"*), so the Docker-on-macOS Groth16 path is inferred from issue threads, not documented — test it before relying on it. RISC Zero's install page likewise states no separate CPU-vs-GPU proving requirements. Peak-RAM figures for either prover on aarch64 macOS are published nowhere I could find; you will have to measure them.

Sources: [SP1 install](https://docs.succinct.xyz/docs/sp1/getting-started/install) · [SP1 hardware acceleration](https://docs.succinct.xyz/docs/sp1/generating-proofs/hardware-acceleration) · [SP1 cycle tracking](https://docs.succinct.xyz/docs/sp1/optimizing-programs/cycle-tracking) · [SP1 precompiles](https://docs.succinct.xyz/docs/sp1/optimizing-programs/precompiles) · [SP1 proof types](https://docs.succinct.xyz/docs/sp1/generating-proofs/proof-types) · [sp1#2289](https://github.com/succinctlabs/sp1/issues/2289) · [sp1#2287](https://github.com/succinctlabs/sp1/issues/2287) · [sp1#841](https://github.com/succinctlabs/sp1/issues/841) · [RISC Zero install](https://dev.risczero.com/api/zkvm/install) · [RISC Zero precompiles](https://dev.risczero.com/api/zkvm/precompiles) · [RISC Zero profiling](https://dev.risczero.com/api/zkvm/profiling) · [RISC Zero local proving](https://dev.risczero.com/api/generating-proofs/local-proving) · [qrypta-pu](https://github.com/ebulgin/qrypta-pu) — all accessed 2026-07-23.