# PQ-STARK-BENCH — Project Kickoff Brief

**Paste this entire file as the first message of a new Claude Code session, or drop it into the repo root as `CLAUDE.md` / `KICKOFF.md`.**

---

## 1. Mission Statement

Build the **first public, reproducible, cross-scheme and cross-prover benchmark suite and reference design for post-quantum signature verification in STARK/zkVM settlement**, with amortized per-transaction costs measured against classical (ECDSA/Ed25519) baselines.

One-sentence pitch: *"Individual proofs of concept exist - Falcon in a Starknet STARK, Dilithium in SP1 - but nobody has published the full matrix: multiple PQ schemes, multiple provers, batch-size scaling curves, and dollar-cost-per-transaction, all reproducible. We are the comparison layer."*

**Positioning rules (result of the July 2026 literature check - do not regress on these):**
- NEVER claim "first PQ signature verification in a STARK." BTQ + StarkWare did that on Starknet in 2024. Our novelty is the cross-scheme, cross-prover, reproducible benchmark MATRIX and its economics, not any single verification.
- Cite prior art prominently and generously (see §3b). Acknowledging s2morrow and Dilithium-ZK is what makes this credible to cryptographers.
- The claim survives being scooped: if s2morrow publishes Stwo numbers, we incorporate and contextualize them - the independent comparison layer becomes MORE valuable, not less. Frame accordingly everywhere (README, site hero, docs).

### What this project IS
- A rigorous measurement project: hard numbers, reproducible on anyone's machine
- A reference architecture document backed by those numbers
- A public dashboard (static site on Netlify, deployed from GitHub) visualizing every result
- A credibility artifact: something citable that positions the author as the reference point for PQ rollup benchmarking

### What this project is NOT (hard rules - never violate)
- **We do NOT invent new cryptographic primitives.** Ever. Only NIST-standardized or NIST-round schemes with existing vetted implementations.
- **We do NOT claim "unbreakable" or "quantum-proof."** Language is always "post-quantum," "quantum-resistant per NIST standardization," with assumptions stated.
- **We do NOT build anything that holds real money.** Prototype and benchmark only.
- **We do NOT fabricate or extrapolate numbers.** Every published figure comes from an actual run, with hardware, versions, and methodology disclosed. If a benchmark fails or is infeasible, we say so.
- **We do NOT overclaim novelty.** Every "first" on the site or README must be scoped exactly (cross-scheme, cross-prover, reproducible, public) and defensible against §3b.

---

## 2. Operator Context (who you're working with)

- The project owner is a founder, **not a professional coder**. You (Claude Code) own all implementation. Explain decisions briefly in plain language; don't ask him to write or debug code.
- He is **visual**: results must always end up on the dashboard, not just in terminal output.
- Communication style: terse, direct, execution-ready. No em dashes in any user-facing text or docs - use hyphens.
- Budget: ~$0 infra. Free tiers only (GitHub free, Netlify free, local/CI compute).

---

## 3. Technical Background (read before coding)

### The core insight being benchmarked
In a validity-proof (rollup-style) architecture:
1. Users sign transactions with post-quantum signatures (large: 666 B - 17 KB)
2. A prover verifies N signatures **inside a STARK circuit**
3. Only ONE proof (~50-200 KB, quantum-resistant since STARKs are hash-based) is posted to the settlement layer
4. Therefore signature SIZE stops mattering for chain storage - **prover TIME and COST become the real metric**, and that metric is unpublished territory

### Schemes to benchmark (all vetted, all have reference implementations)
| Scheme | Type | Sig size | Status | Notes |
|---|---|---|---|---|
| ECDSA secp256k1 | Classical | ~71 B | Baseline | What Bitcoin/Ethereum use |
| Ed25519 | Classical | 64 B | Baseline | Modern classical baseline |
| ML-DSA-44 (Dilithium2) | Lattice | ~2,420 B | NIST FIPS 204 | The default PQ standard |
| Falcon-512 (FN-DSA) | Lattice | ~666 B | NIST selected | Smallest standardized; tricky impl (FP Gaussian sampling) - use reference impl only, never reimplement |
| SLH-DSA-128s (SPHINCS+) | Hash | ~7,856 B | NIST FIPS 205 | Conservative; hash-based = cheap inside STARK circuits potentially |
| SLH-DSA-128f (SPHINCS+) | Hash | ~17,088 B | NIST FIPS 205 | Fast-signing variant |

### Libraries (do not hand-roll crypto - bind to these)
- **Native benchmarks:** `liboqs` (Open Quantum Safe) via its C library + language bindings, or Rust crates `pqcrypto-*` (PQClean-based), `fips204`, `fips205`. For classical: `secp256k1`, `ed25519-dalek`.
- **In-circuit benchmarks - two tracks:**
  - **Track A (primary, pragmatic): zkVM route.** Run signature verification as guest code inside **BOTH RISC Zero AND SP1 (Succinct)**. Both provers are MANDATORY, not optional - the cross-prover comparison is a core novelty claim (see §1 and §3b). Compiling existing verification code to zkVM guests gives real prover-cost numbers in days, not months.
  - **Track B (later, optimization): hand-written STARK circuits** with **Winterfell** (Rust) or Stone. Only attempt after Track A ships. Expect Track B to be 10-100x faster than zkVM but 10x the engineering effort. If s2morrow's Cairo/Stwo numbers publish, incorporate them here as a third prover column instead of reimplementing.
- Pin every version. Record commit hashes in results.

### 3b. Verified prior art (literature check performed July 23, 2026 - seed LITERATURE.md with these, then re-verify links and check for updates in session 1)

| Work | What it is | What it means for us |
|---|---|---|
| BTQ + StarkWare Falcon verification (2024) | First STARK-based verification of a Falcon signature, on Starknet | Kills any "first PQ-in-STARK" claim. Cite as the founding proof of concept. https://www.btq.com/blog/completing-the-first-falcon-signature-verification-in-starkware-initiating-the-transition-to-a-quantum-safe-ethereum |
| s2morrow (starkware-bitcoin org, active) | Falcon-512 + SPHINCS+ verification in Cairo/Stwo; goal is proving-time comparison and benchmarks vs approaches like LaBRADOR; roadmap item "Stwo proving benchmarks" UNCHECKED as of July 2026 | Closest competitor; has code, has NOT published numbers. Watch this repo every few weeks. If they publish, incorporate their numbers as a comparison column and adjust claims. https://github.com/starkware-bitcoin/s2morrow |
| Dilithium-ZK on SP1 (indie, Dec 2025) | ML-DSA-65 verification in SP1 with NTT gadget; published: ~22s proof, 260-byte on-chain verification | One cell of our matrix already exists. Cite it, reproduce it, extend to batching + other schemes. https://dilithium-zk-landing.vercel.app/ |
| Native PQ benchmarks (NIST Sigs Zoo, pqm4, academic testbeds) | Extensive published sign/verify/keygen numbers across schemes | Phase 1 is reproduction for baseline continuity, NOT a novelty claim. Cross-check our native numbers against these; large deviations mean our harness is wrong. https://pqshield.github.io/nist-sigs-zoo/ |
| "Lattice" paper (arXiv 2603.07947, Mar 2026) | PQ-only chain posting ML-DSA-44 signatures naively on-chain, no STARK layer; includes native perf and tx-size analysis | Perfect real-world example of the "naive on-chain" model our amortization curves compare against. Cite in methodology. |
| Ethereum/Starknet PQ roadmaps (2025-2026) | StarkWare announced a Starknet post-quantum roadmap; ecosystem interest is rising | Tailwind: the audience for these benchmarks is growing. Cite for motivation, verify current status when writing. |

**Standing rule:** re-run the literature search (queries: "Falcon verification zero-knowledge", "Dilithium zkVM benchmark", "post-quantum signature STARK aggregation", "s2morrow") at the start of any session that will publish claims, and log findings with dates in LITERATURE.md.

---

## 4. Deliverables and Phases

### Phase 1 - Native benchmark suite (first sessions)
Rust workspace crate `bench-native`:
- For every scheme in the matrix: keygen time, sign time, verify time, batch-verify time (where supported), public key size, signature size
- A defined canonical transaction format (see §5) → compute **bytes-per-transaction** for each scheme in a "naive on-chain" model
- Statistical rigor: use `criterion` for micro-benchmarks; report median + p95, N ≥ 100 iterations, warmup; record CPU model, RAM, OS, rustc version into every results file
- Output: machine-readable `results/native/<timestamp>-<host>.json` conforming to the schema in §6

### Phase 2 - In-circuit benchmark suite (the novel contribution)
Crate `bench-zkvm`:
- Guest programs for BOTH RISC Zero AND SP1 (both mandatory - cross-prover is a core claim) that verify 1, 10, 100, 1000 signatures per proof, for: Ed25519 (baseline), ML-DSA-44, Falcon-512, SLH-DSA-128s
- Measure: prover wall time, prover cycles, peak RAM, proof size, verifier time
- Compute the headline metric: **amortized proving cost per transaction vs batch size**, and the crossover point where PQ-in-STARK beats naive PQ-on-chain economics
- Reality check to include: convert prover time → estimated $ cost using current cloud GPU/CPU pricing (fetch current prices, cite source and date)
- If Falcon's floating-point sampling makes guest compilation infeasible, benchmark Falcon **verification only** (verification is integer-friendly); document any scheme that cannot run in-guest and why - a negative result is still a publishable result

### Phase 3 - Public dashboard (build the shell EARLY - session 1 or 2 - even with placeholder data, owner is visual)
Static site in `/site`:
- Stack: **Vite + React + TypeScript + TailwindCSS + Recharts**. No backend - the site reads the JSON files from `results/` at build time
- Pages/sections:
  1. **Hero:** the one-line thesis + headline number once available ("PQ signature verification at $X per 1,000 tx")
  2. **Signature Size Comparison:** bar chart, log scale, all schemes - the "why this matters" visual
  3. **Native Performance:** sign/verify times, sortable table + charts
  4. **In-Circuit Results:** amortized cost vs batch size (line chart, one line per scheme) - THE money chart
  5. **Methodology:** hardware, versions, how to reproduce (`git clone && just bench`)
  6. **Honest Limitations:** what these numbers do and don't show
- Design: dark theme, technical/credible aesthetic (think research-lab, not landing page). Read `ui-ux-pro-max` / `frontend-design` skills if available in the environment before styling. Mobile-responsive. No em dashes anywhere in copy - hyphens only.
- Deploy: **GitHub Actions → Netlify** on every push to `main`. Netlify MCP/connector is available for setup. Also wire a scheduled/manual Action that runs the native benchmark suite in CI (label CI results as CI-hardware, distinct from local runs).

### Phase 4 - Reference design writeup (after numbers exist)
`docs/REFERENCE-DESIGN.md`: architecture for a PQ-account + STARK-settlement chain justified by the measured numbers. Threat model, key rotation, hybrid (classical+PQ) signature option, recovery-layer hooks (owner's second priority - design stub only, not implemented).

---

## 5. Canonical Transaction Format (for bytes-per-tx math)

Define once in `bench-native/src/tx.rs` and reuse everywhere:

```
Transaction {
  version:      u8
  nonce:        u64
  from_pubkey:  [scheme-dependent]   // or 32-byte key-hash address + pubkey revealed in witness
  to_address:   32 bytes
  amount:       u64
  fee:          u64
  payload_hash: 32 bytes (optional, zeroed)
  signature:    [scheme-dependent]
}
```

Report bytes-per-tx BOTH ways: (a) pubkey-in-tx, (b) address-only with pubkey in witness data. Note that model (b) is what serious designs use.

---

## 6. Results JSON Schema (site consumes this - keep stable)

```json
{
  "run_id": "2026-07-23T10-00-00Z-hostname",
  "kind": "native | zkvm",
  "environment": {
    "cpu": "", "cores": 0, "ram_gb": 0, "os": "",
    "rustc": "", "library_versions": {"liboqs": "", "risc0": ""},
    "is_ci": false
  },
  "results": [
    {
      "scheme": "falcon-512",
      "operation": "verify | sign | keygen | prove_batch",
      "batch_size": 1,
      "median_ns": 0, "p95_ns": 0, "iterations": 100,
      "sig_bytes": 0, "pubkey_bytes": 0, "proof_bytes": null,
      "prover_cycles": null, "peak_ram_mb": null
    }
  ]
}
```

---

## 7. Repo Structure

```
pq-stark-bench/
├── KICKOFF.md              (this file)
├── CLAUDE.md               (working agreements - generate from §8)
├── README.md               (public-facing: thesis, headline results, reproduce instructions)
├── justfile                (just bench-native / just bench-zkvm / just site-dev / just site-build)
├── crates/
│   ├── bench-native/
│   ├── bench-zkvm/
│   └── tx-format/
├── results/
│   ├── native/*.json
│   └── zkvm/*.json
├── site/                   (Vite + React dashboard)
├── docs/
│   ├── METHODOLOGY.md
│   ├── LITERATURE.md       (prior-art survey from session 1, with links and dates)
│   └── REFERENCE-DESIGN.md (phase 4)
└── .github/workflows/
    ├── bench.yml           (manual/scheduled CI benchmark runs)
    └── deploy-site.yml     (build + deploy to Netlify on push to main)
```

---

## 8. Working Agreements (put these in CLAUDE.md)

1. **Session 1 order:** (a) literature check with fresh web searches → write `docs/LITERATURE.md`; (b) scaffold repo + justfile + CI; (c) get ONE scheme (Ed25519) benchmarked end-to-end into JSON; (d) scaffold the site rendering that one JSON. A thin vertical slice first - owner sees a live chart from day one.
2. Every session ends with: code committed and pushed, site deploy green, and a 3-line plain-language summary of what changed and what's next.
3. Never fabricate a number. Missing data renders as "not yet measured" on the site.
4. Prefer Rust for benchmark code, TypeScript for the site. Pin toolchains (`rust-toolchain.toml`).
5. All copy (site, README, docs) uses hyphens, never em dashes.
6. Any claim about other projects (Algorand, Starknet, papers) gets verified by web search at time of writing and linked in LITERATURE.md with an access date.
7. If a task hits a wall (e.g., Falcon won't compile to a zkVM guest), document the wall in METHODOLOGY.md and move on - negative results are results.
8. Security posture: this is measurement code, but still no `unsafe` without justification, no custom crypto, dependencies from crates.io with meaningful download counts only.

---

## 9. Success Criteria

- [ ] Reproducible: a stranger can `git clone`, run `just bench-native`, and get the same shape of numbers
- [ ] The money chart exists: amortized prover cost per tx vs batch size, PQ schemes vs Ed25519 baseline
- [ ] Live dashboard on Netlify, auto-deployed from `main`
- [ ] LITERATURE.md honestly positions the work (first, or best reproduction - whichever is true)
- [ ] Zero invented cryptography, zero unverifiable claims
- [ ] A README a cryptographer could read without wincing

## 10. Local Skills (install at session start)

The repo ships with two project-level skills under `.claude/skills/` (extract `claude-skills-install.zip` into the repo root if not already present - it creates `.claude/skills/cryptography/` and `.claude/skills/qiskit/`). Claude Code auto-discovers project skills at this path; verify with `/skills` or by listing the directory in session 1.

**cryptography** - classical crypto hygiene reference (approved algorithms, key sizes, deprecated-algorithm checklist, parameter validation). Consult it when writing any code that touches classical primitives: the ECDSA/Ed25519 baselines, hashing in the tx format, and QA passes on benchmark code. Its "never roll your own crypto" rule aligns with §1 hard rules - enforce it.

**qiskit** - quantum circuit toolkit (IBM Qiskit). **Scope note, important:** this project is post-quantum *cryptography*, which is classical math - Qiskit and quantum circuits are NOT needed for any benchmark in this brief, and no phase should pull it in by default. It is installed only for possible future side-work (e.g., illustrating Shor's-algorithm threat context on the dashboard's methodology page). Do not let its presence steer the roadmap toward quantum-computing experiments.

## 11. First Message to Send After Pasting This

"Read this brief fully, especially §1 positioning rules and §3b verified prior art. Then: 1) verify both local skills in .claude/skills/ are discovered, 2) re-verify the §3b prior-art links, check s2morrow for newly published numbers, and write docs/LITERATURE.md before writing any code, 3) scaffold the repo and get the Ed25519 vertical slice + placeholder dashboard live, with the site hero using the corrected cross-scheme cross-prover framing. Go."
