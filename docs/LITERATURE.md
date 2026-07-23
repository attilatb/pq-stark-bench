# Literature and Prior Art

## 1. Method and scope of this survey

This document records what is already published about verifying post-quantum (PQ) signatures inside STARK / zkVM proof systems, and what PQ-STARK-BENCH adds on top of it. It exists so that no claim in this project rests on an assertion nobody checked.

Rules applied to every entry below:

1. Every external claim carries an inline link and the words "accessed 2026-07-23". Every source in this file was fetched on 2026-07-23 unless the entry says otherwise.
2. Numbers are quoted verbatim with the units the source printed. Where a figure is not published, the text says "not published". No figure in this document was rounded, reconstructed, or recalled from memory.
3. Three provenance classes are kept distinct and are labelled in place: **measured-and-published** (the source ran it and printed the number), **vendor-claimed** (the source asserts it without a reproducible artifact), and **our inference** (arithmetic or reasoning we did on top of published figures).
4. Where our own verification pass **contradicted** a claim we had been handed, or could not confirm it, the contradiction is stated in the body text rather than quietly dropped. Section 10 is the full verification log.
5. Sources that 404, are paywalled, or do not say what they are cited as saying are called out explicitly. Those are treated as results, not as gaps to paper over.

Scope boundary. This survey covers signature **verification** inside a proof system, plus the native and naive-on-chain baselines needed to interpret it. It does not attempt to survey lattice-based proof systems generally, PQ key exchange, or PQ TLS.

A methodological note that constrains everything downstream: page-summarising fetchers repeatedly misread tabular repository content during this survey. In one case a summariser reported a roadmap checkbox as complete when the raw file says `- [ ]` (see s2morrow, section 3). In another it silently dropped two rows from a benchmark table, including the single most important row (see OpenZeppelin, section 5). Every table and checkbox quoted in this document was pulled from a raw file or an API response, not from a rendered-page summary.

## 2. Prior art: PQ signature verification inside a STARK is already done

This is the first section on purpose. PQ-STARK-BENCH does not claim to be the first to verify a post-quantum signature inside a STARK, and any framing that implies otherwise is wrong and easily refuted.

### 2.1 The earliest work we could find: IACR ePrint 2021/1048 (AsiaCCS 2022)

Khaburzaniya, Chalkias, Lewi and Malvai, "Aggregating and thresholdizing hash-based signatures using STARKs", AsiaCCS 2022, Nagasaki, Japan, 30 May to 3 June 2022, DOI [10.1145/3488932.3524128](https://doi.org/10.1145/3488932.3524128); preprint [IACR ePrint 2021/1048](https://eprint.iacr.org/2021/1048), received 2021-08-16, revised 2022-03-14 (accessed 2026-07-23).

This paper verifies post-quantum (hash-based) signatures inside a STARK and publishes prover benchmarks, roughly two years before any Falcon-in-Cairo work. Table 3 gives prover time, prover RAM and aggregate-signature size at batch sizes n = 128 / 256 / 512 / 1024 (aggregate) and n = 127 / 255 / 511 / 1023 (threshold), at two STARK security levels, on named hardware: "We ran benchmarks on an 8-core Intel Core i9 processor @ 2.4 GHz with 32 GB of RAM."

Measured-and-published, aggregate rows, 96-bit STARK security (prover time / prover RAM / signature size):

| n | prover time | prover RAM | signature size |
|---|---|---|---|
| 128 | 2.5 sec | 0.9 GB | 68 KB |
| 256 | 5.1 sec | 1.8 GB | 71 KB |
| 512 | 10.5 sec | 3.7 GB | 77 KB |
| 1024 | 19.7 sec | 7.4 GB | 83 KB |

At 123-bit STARK security the same rows read 3.2 sec / 1.2 GB / 129 KB, 6.7 sec / 2.4 GB / 140 KB, 13.6 sec / 4.8 GB / 155 KB, 25.7 sec / 9.5 GB / 165 KB.

Five things about this paper matter to us, and three of them correct claims we were handed.

- **The scheme is not a NIST scheme.** It is Lamport+, described in the paper as "a WOTS instance for w = 2 (Winternitz parameter)", instantiated with the Rescue-Prime hash over a 128-bit prime field, chosen explicitly for "its efficient encoding in ZKP systems". That is the opposite of the constraint a FIPS scheme imposes: ML-DSA and SLH-DSA are locked to SHAKE256 / SHA-2, which are hostile to a prime-field AIR. The paper's Table 4 mentions Falcon-512 and Dilithium3 only as concatenated-size comparisons; neither is verified in-circuit.
- **The security levels are proof-soundness levels, not signature strength.** Table 1 sets 96-bit vs 123-bit by FRI query count (27 vs 34 queries). Lamport+ itself "targets 123 bits of security". A chart labelled "96-bit" without that footnote reads as a weakened signature, which it is not.
- **The paper does not publish amortized per-signature cost.** The strings "amortiz", "per signature" and "per-signature" do not appear in the 15-page PDF. The commonly quoted 19.5 / 19.9 / 20.5 / 19.2 ms per signature are **our inference** from dividing Table 3's 96-bit column by n (19.531, 19.922, 20.508, 19.238 ms). They must never be attributed to the authors.
- **CONTRADICTION we found.** We were told the code is "asserted to be in facebook/winterfell". The paper does not say that. Its artifact reference [6] reads verbatim: "A. authors. STARK prover and verifier (Rust implementation). https://github.com/anonauthorsub/asiaccs_2021_440" - a leftover anonymised submission URL that returns HTTP 404 today (checked via the GitHub API, 2026-07-23). The [facebook/winterfell](https://github.com/facebook/winterfell) attribution is **our inference**, from shared authorship (top contributor `irakliyk`, commit email `irakliy81@gmail.com`, byte-identical to the front page), matching examples (`examples/src/lamport/{aggregate,threshold}`), matching instantiation (Rescue-Prime, 32-byte public key, 8 KB signature, 123-bit target) and matching hardware class. A GitHub code search across `repo:facebook/winterfell` for "asiaccs OR Khaburzaniya OR 3524128" returns zero hits: neither artifact cites the other.
- **Winterfell publishes its own, different table for the same construction.** Its README benchmarks Lamport+ at 123-bit on "Intel Core i9-9980KH @ 2.4 GHz and 32 GB of RAM using all 8 cores": n = 1024 gives 3.2 sec trace / 20.5 sec proving / 7.6 GB / 152 KB proof / 5.9 ms verify, against the paper's 25.7 sec / 9.5 GB / 165 KB for nominally the same point. Two published measurements of the same thing disagree. Never blend the two tables into one chart. (Repo last pushed 2025-07-19, latest release v0.13.1; accessed 2026-07-23.)

The Winterfell README also states the scaling result plainly: "Trace time and prover RAM ... grow pretty much linearly with the size of the computation", "Proving time grows very slightly faster than linearly", "Proof size and verifier time grow much slower than linearly (actually logarithmically)". **Flat amortization of prover cost is prior art, not a discovery.**

Access note, and a project-wide infrastructure problem: `https://eprint.iacr.org/2021/1048.pdf` returns HTTP 403 behind a Cloudflare interstitial to automated fetchers (multiple user-agent and referer variants tried). The abstract page fetches fine. We read the PDF from the [Wayback snapshot of 2026-01-21](https://web.archive.org/web/20260121180642if_/https://eprint.iacr.org/2021/1048.pdf) (674,241 bytes, 15 pages). The ACM version is closed access (Semantic Scholar `openAccessPdf` status "CLOSED"). Every ePrint PDF citation in this project should carry a Wayback fallback, or reviewers cannot reproduce our reads.

### 2.2 BTQ and StarkWare, Falcon on Starknet: 2023, not 2024, and narrower than advertised

BTQ, "Completing the First Falcon Signature Verification in Starkware - Initiating the Transition to a Quantum-Safe Ethereum", visible date on page **Sep 18, 2023** ([btq.com](https://www.btq.com/blog/completing-the-first-falcon-signature-verification-in-starkware-initiating-the-transition-to-a-quantum-safe-ethereum), accessed 2026-07-23).

We were handed this as "BTQ + StarkWare did Falcon on Starknet in 2024". **Four corrections, one of them load-bearing.**

1. **The date is wrong.** The page says Sep 18, 2023. Publishing "2024" is off by a year and visible to anyone who clicks.
2. **The claim is narrower in venue and broader in scheme than usually reported.** BTQ's actual words are "the first successful STARK-based verification of post-quantum digital signatures **on Starknet**", and "This achievement marks the first **step** in supporting NIST standardized post-quantum digital signature algorithms on Ethereum". Self-asserted on a vendor blog, not third-party verified.
3. **The unqualified "first" is false.** ePrint 2021/1048 (section 2.1) verified post-quantum signatures inside a STARK two years earlier, with benchmarks. BTQ's sentence survives only under the reading "first on Starknet" or "first for a lattice scheme in Cairo".
4. **It is not established that a proof was produced.** The page shows an image captioned "Cairo profile of Falcon signature verification" and says the work "is based on our prior work on quantum-safe signature aggregation using PQScale". It never states that a STARK proof was generated, submitted, or verified on mainnet or testnet. What is supported is "a Cairo program was written and profiled".

The Falcon parameter set is **not published** on the page (Falcon-512 is never mentioned). **No proving time, no Cairo step count, no constraint count, no RAM, no proof size, no gas.** The only numbers are aggregation claims tied to PQScale, all vendor-claimed and unbenchmarked: "12.5x" space savings, an aggregate "ten times smaller than Falcon signatures", "several thousands of Falcon signatures in a proof", and a claim that PQScale brings "the per-signature cost of Falcon to below the cost of ECDSA".

That absence strengthens our position rather than weakening it: the empty cell is not only ML-DSA, it is **Falcon-in-STARK prover cost too**.

Note also that the "first" framing has been laundered into third-party literature. The arXiv survey "Quantum Disruption" ([arxiv.org/html/2512.13333v1](https://arxiv.org/html/2512.13333v1), accessed 2026-07-23) repeats it verbatim as "BTQ, in collaboration with StarkWare, demonstrated the first verification of FALCON signature on StarkNet", citing no primary evidence beyond the BTQ blog and citing no earlier STARK or SNARK PQ-signature work at all. It is evidence that the survey literature has the gap we are filling; it is not independent confirmation of the "first".

### 2.3 HAPPIER: hash-based PQ aggregation inside RISC Zero, 2025

Saygan, Gündoğan, Arslan, Gönen, "HAPPIER: Hash-Based, Aggregatable, Practical Post-quantum Signatures Implemented Efficiently with Risc0", LightSec 2025 (Istanbul, 1 to 2 September 2025), LNCS 16216, first online 31 January 2026, [DOI 10.1007/978-3-032-15541-2_1](https://link.springer.com/chapter/10.1007/978-3-032-15541-2_1) (accessed 2026-07-23).

**PAYWALL, said loudly: we could not read the paper.** Unpaywall returns verbatim `{"is_oa": false, "oa_locations": [], "oa_status": "closed"}` for that DOI (accessed 2026-07-23). There is no ePrint preprint, no arXiv version; Semantic Scholar returns a null abstract; ResearchGate 403s. **Every statement below about method comes from the public code artifact, not the paper.** Whether the chapter contains a memory column, a hardware description, or a true non-extrapolated end-to-end timing is a known unresolved limitation of this survey.

This is the most directly adjacent prior work to PQ-STARK-BENCH's headline chart: a hash-based PQ signature aggregated inside **RISC Zero**, a general-purpose zkVM, with a prover-time-vs-batch-size sweep. Code and raw logs: [ArdaSaygan/PQ-Aggregation-via-Recursive-SNARKs](https://github.com/ArdaSaygan/PQ-Aggregation-via-Recursive-SNARKs), last commit `be73fdb` 2025-04-18, pinned to `risc0-r0vm` 1.2.5 (accessed 2026-07-23).

Abstract, verbatim: "existing schemes remain impractical due to high memory usage and slow aggregation. While prior solutions produce smaller signatures (hundreds of KB), they require hundreds of gigabytes of RAM to aggregate 2^10 signatures. In contrast, our scheme generates slightly larger signatures (2-3 MB) but drastically reduces aggregation time and can handle up to 2^16 signatures on a standard laptop".

Findings from the artifact that materially qualify that abstract:

- **The scheme is not RFC 8391 XMSS.** `data-types/src/lib.rs` imports `hashsig::signature::generalized_xmss::GeneralizedXMSSSignatureScheme` with the comment "Copied from https://github.com/b-wagn/hash-sig/...", parameters `LOG_LIFETIME 16`, `CHUNK_SIZE 4`, `HASH_LEN 24`, Winternitz encoding. Cite it as generalized XMSS, not XMSS.
- **The headline times are extrapolated, not wall-clock.** `merge2.rs:188-190` computes `aggregation_time = (duration_sig_to_proof1 + duration_sig_to_proof2) / 2 + duration_merge_proofs * no_agg_signatures.ilog2() / merging_proofs.ilog2()`. That is one leaf proof plus one merge per tree level: an idealised infinitely-parallel critical path.
- **Every run aggregates exactly two distinct signatures.** `merge4.rs:145-148` sets `acc_3 = acc00.clone(); acc_4 = acc01.clone();` and every log line reads `Bitfield: 0b00000011` regardless of N. The batch-size axis is a build-time constant that widens the in-guest bitfield.
- **No memory figure and no hardware spec exist anywhere in the artifact.** A grep for `peak memory|max rss|memory usage|[0-9] ?GB|[0-9] ?GiB` across all `.rs`, `.md` and `.txt` returns zero hits. The abstract attacks prior work on RAM while publishing none of its own. "A standard laptop" is never pinned to a CPU.
- **The 2-3 MB claim does not match most of the artifact.** Only the XMSS256-192 merge2 configuration lands in range (2,404,188 bytes at N=8192; 3,230,448 bytes at N=32768). The `FinalBenchmark` sweep the README points at reports 7,148,152 to 7,987,660 bytes for merge2, 13,457,820 to 15,655,740 for merge4, and 27,637,964 to 30,299,144 for merge8. We cannot adjudicate without the paper.
- Two cells of their own sweep are missing: `run_test.sh` requests `merge4_sig9192` and `merge8_sig9192`; neither file exists. ("9192" is itself an apparent typo for 8192 in their script.)

Representative published figures, verbatim from the logs (extrapolated totals marked as such): merge2 N=128 "Estimated time to aggregate all signatures ... 4527.254592 seconds", proof size 7,390,292 bytes; merge2 N=1024 leaves 552.05362175 s and 558.525891375 s, merge 570.45527625 s, verify 0.440504 s, estimated total 6259.842519062 s; merge4 N=262144 estimated total 11021.879351913 s, proof 30,273,452 bytes. One RISC Zero session log gives "5767168 total cycles", "4221595 user cycles (73.20%)", "1390770 paging cycles (24.12%)", "4726 Sha2 calls, 349724 cycles, (6.06%)".

**Consequence for our positioning, stated bluntly.** Between ePrint 2021/1048 and HAPPIER, the claim "first prover-cost-vs-batch-size measurement for a PQ signature in a zkVM" is dead. Do not make it.

## 3. Closest active work, with exact current status

### 3.1 starkware-bitcoin/s2morrow: same stated goal as ours, benchmarks still unchecked

[starkware-bitcoin/s2morrow](https://github.com/starkware-bitcoin/s2morrow), "STARK-based signature aggregation for Falcon and SPHINCS+", MIT, "Copyright (c) 2025 Michael Zaikin". Repo created 2025-04-27T17:35:58Z; last commit on `master` [`831bb51`](https://github.com/starkware-bitcoin/s2morrow/commit/831bb518b06df614462e8883e1e2c5d627ec437e) 2026-03-23T11:39:00Z, by Michael Zaikin `<michael.zaikin@starkware.co>`. 21 commits, 16 stars, 7 forks, two open draft PRs both stale since 2026-02-03 (accessed 2026-07-23).

The README states our exact goal, verbatim: "This project explores zkVM approach for batch verification of multiple PQ signatures. The goal is compare proving time for different signature schemes, as well as benchmark vs other approaches (e.g. LaBRADOR) in terms of proof size (compression ratio) and verification time." Stack: "ZKVM: Cairo / STARK prover: Stwo".

Roadmap, verbatim from the raw README on `master` (blob `ba24bb8`):

```
- [x] Falcon512 verification
- [x] Sphincs+ 128s with SHA2 (simple mode) verification
- [ ] Stwo proving benchmarks

Follow-up:
- [x] Sphincs+ 128s with Blake2s and 4-byte aligned address encoding
- [ ] Falcon512 with probabilistic polynomial multiplication checking
```

**No proving time, no memory, no proof size, and no batch-size figures are published, as of accessed 2026-07-23.** The repo contains proving infrastructure and zero results files. The only numbers in it are prover configuration, verbatim from `prover_params.json`: `"channel_hash": "blake2s"`, `"pow_bits": 26`, `"log_last_layer_degree_bound": 0`, `"log_blowup_factor": 1`, `"n_queries": 70`. The Makefile's `falcon-args` generates `--num_signatures 1`: **a batch-size sweep is not wired up at all.**

**Process disconfirmation, flagged loudly.** A page-summarising fetch of the repo returned "**Completed:** ... ✅ Stwo proving benchmarks". That is false; the raw README says `- [ ]`. Anyone sourcing competitor status from a rendered summary will publish a wrong claim about the closest active work. Note also that `raw.githubusercontent.com/.../main/README.md` returns HTTP 404 here because the default branch is `master`.

**Reproducibility blocker.** Every `*-prove` Makefile target depends on `cargo +nightly-2025-07-14 install --git ssh://git@github.com/m-kus/proving-utils.git --rev efbaeebfdce3463aa61e16d7d8e6069f03df0994 stwo_run_and_prove --force`. That SSH URL does not resolve publicly. The Cairo verifier code is reproducible; their proving pipeline is not, off the shelf.

Correctness caveat on their SPHINCS+ Blake2s variant: issue #6 (opened 2026-02-02, closed 2026-03-23) reports "The `blake2s` hasher ignores most hash operations, and only hashes the suffixes of input messages ... this means that the Cairo package incorrectly verifies SPHINCS+ signatures". Any Blake2s cost figure measured between 2025-12-19 and 2026-03-23 would be drastically understated. Their own benchmark scripts predate the fix.

Also verified: `docs.swmansion.com/maat/nightly-nightly-2026-04-18-.../s2morrow-831bb518b/logs.txt`, a third-party CI run against that exact commit which would have carried real resource-usage output, **returns HTTP 404** (accessed 2026-07-23). The nightly directory has rotated. Do not cite it.

### 3.2 The name collision: feltroidprime/s2morrow and s2morrow.xyz

Web search conflates the StarkWare repo above with [feltroidprime/s2morrow](https://github.com/feltroidprime/s2morrow), a fork by a different author (~208 commits, MIT, last commit `4eff9ab` 2026-03-05), with a live site [s2morrow.xyz](https://www.s2morrow.xyz/) titled "Falcon-512 | Post-Quantum Signatures on Starknet" (accessed 2026-07-23).

The fork **does** publish numbers, but they are Cairo VM step counts and Starknet L2 gas for on-chain verification of **one** signature. They are not prover time, not batched, not amortized. Verbatim from its README: "Profiled at commit `a1f5ed9` with `cairo-profiler` (cumulative steps): `verify` (e2e) | 63,177 ... `verify_with_msg_point` | 26,301 ... `hash_to_point` | 5,988 ... Total test cost: 119,034 steps ... L2 gas: `verify` ~13.2M". Site figures: "~9.5M L2 gas", "17x calldata compression (1,030 felts reduced to 62 felts on-chain)", "29 felt252 slots for 512 Zq coefficients", "65% less gas than secp256r1 syscall", "2x the cost of garaga ECDSA". No page date is visible on the site.

Three warnings. **The fork's own two published Falcon-512 `verify` gas figures disagree with each other**: README "~13.2M" vs site "~9.5M", with no date or commit given for the site number. The fork dropped SPHINCS+ and aggregation entirely, and swapped SHAKE-256 for Poseidon in hash-to-point, so it is **not comparable to any FIPS-conformant benchmark**. And it is not StarkWare's work, despite being surfaced as such.

### 3.3 Dilithium-ZK / sp1-ntt-gadget: does not fill the ML-DSA cell

Kota (@phillyj1026), "Building a Zero-Knowledge Verifier for Dilithium Signatures: NTT Gadget Implementation in SP1 zkVM", Medium, **8 Dec 2025** ([article](https://medium.com/@phillyj1026/building-a-zero-knowledge-verifier-for-dilithium-signatures-ntt-gadget-implementation-in-sp1-6c50ab262836)); [landing page](https://dilithium-zk-landing.vercel.app/); crate [`sp1-ntt-gadget` v0.1.0](https://crates.io/crates/sp1-ntt-gadget), published 2025-12-08T05:58:11Z, 39 total downloads (all accessed 2026-07-23).

The pages load. The claims are ML-DSA-**65** (NIST Level 3), not ML-DSA-44: the crate description reads "NTT/INTT Custom Gadget for Dilithium (ML-DSA-65) verification in SP1 zkVM with 60-bit soundness PIC".

Headline vendor-claimed figures: "Proof time: 22.07 seconds", "Proof size: 260 bytes", "Cost per proof: ~$0.045", mode Groth16, on "SP1 Network". Native microbenchmarks, on "Apple M1 hardware": "Forward NTT ~1.3 µs, Inverse NTT ~1.9 µs, Verified NTT ~5.1 µs".

**Five reasons this does not fill the ML-DSA-in-zkVM cell, all verified.**

1. **Scope.** The crate's own README states it is "not a complete ML-DSA signature verifier". It ships an NTT/INTT gadget with a 4-challenge polynomial identity check. The 22.07 s therefore cannot be cited as the cost of verifying an ML-DSA-65 signature. What program produced it is never stated unambiguously.
2. **The numbers do not self-check.** The published cycle breakdown is NTT/INTT "~580,000" cycles "~50%", hashing "~200,000" "~18%", other "~320,000" "~32%", against a stated "Total: 5,625,411" cycles. The parts sum to 1,100,000, off from the stated total by 5.1x; and 580,000 / 5,625,411 = 10.3%, not 50%. Both figures are published side by side.
3. **Hardware is not published.** 22.07 s is on SP1 Network, an outsourced proving service with undisclosed, time-varying hardware. The only named hardware (Apple M1) attaches to native microsecond microbenchmarks that are not proving times. Conflating the two would be a serious error.
4. **The linked repository is empty.** [github.com/kota1026/pq-wallet-sp1-ntt-gadget-](https://github.com/kota1026/pq-wallet-sp1-ntt-gadget-) renders "This repository is empty"; the GitHub API returns `"size": 0`, zero branches, and `created_at == pushed_at == updated_at == "2025-12-08T05:53:41Z"` (accessed 2026-07-23). Partial source survives only inside the crates.io tarball, and the manifest carries `exclude = ["tests/*", "examples/*", "scripts/*", ".github/*"]` while declaring `[[example]] name = "prove_benchmark"`. **The programs that produced every headline number, and the claimed "104 tests" / "132,731" fuzz iterations, are in no public artifact.**
5. **260 bytes is not a result of this work.** It is the fixed SP1 Groth16 on-chain proof size that any SP1 program wrapped in Groth16 emits.

Minor but worth recording so nobody repeats it: docs.rs shows "Release Date: June 13, 2026" for this crate. That is a docs rebuild, not the release; the crate was published 2025-12-08. Also, SP1 is pinned here at `sp1-zkvm = "5.0"` / `sp1-sdk = "5.2.3"`, well behind the v6.x line, so a re-measurement on current SP1 would not be comparable to 22.07 s anyway.

Handling rule for this project: cite it as an unreproduced indie datapoint. **Do not put 22.07 s in any chart as an ML-DSA-65 verification cost.**

### 3.4 leanEthereum/leanBench: the methodological sibling, not a competitor

[leanEthereum/leanBench](https://github.com/leanEthereum/leanBench), MIT, "Copyright (c) 2026 lean Ethereum"; repo created 2026-05-06, last commit 2026-06-20T11:03:51Z; dashboard at [bench.leanroadmap.org](https://bench.leanroadmap.org/) (accessed 2026-07-23).

We were told this benchmarks "a post-quantum hash-based signature across six proving machines". **Four corrections.**

1. The scheme is **leanSig**, a purpose-built generalized-XMSS over Poseidon, whose own README calls it "a *prototypical* Rust implementation" that "has *not been audited and is not meant to be used in production*". It is not ML-DSA, SLH-DSA or FN-DSA. leanBench is adjacent to our cell, not overlapping with it.
2. **One prover, not six.** leanBench measures leanVM only (two API feature flags of the same VM). The six names (Binius M3, SP1, KRU, STU, Jolt, OpenVM) live on [leanroadmap.org](https://leanroadmap.org/) as an aspirational track "Post-Quantum Signature Aggregation with zkVMs", progress "50%", whose zkVM milestones are dated Feb 2025 with no linked numbers. It is single-prover, multi-hardware.
3. "Live" overstates it: last commit 2026-06-20, newest result file 2026-06-18, roughly five weeks cold, 1 star, effectively one active contributor.
4. **The dashboard publishes no numbers in fetchable HTML.** It is a client-side Chart.js page. Every figure below comes from the committed `results/*.json`, not the rendered page. There is also no dollar-cost metric anywhere in it.

Measured-and-published, run `2026-06-18T17-09-44Z__934230ae43`, machine `"INTEL(R) XEON(R) PLATINUM 8581C CPU @ 2.30GHz"`, 2 physical cores, 14.6 GB, label `c4-standard-4`, rustc 1.96.0, n = 10 iterations:

| workload | mean_ns | peak rss_bytes | proof_kib_root |
|---|---|---|---|
| aggregate.flat_125_r2 | 1331475651 | 1488793600 | 188 |
| aggregate.flat_250_r2 | 2458346374 | 2171482112 | 196 |
| aggregate.flat_500_r2 | 4869051191 | 3299512320 | 209 |
| aggregate.flat_1000_r2 | 9922054126 | 5908992000 | 229 |

**Our inference** from those rows: 10.65 / 9.83 / 9.74 / 9.92 ms per signature at N = 125 / 250 / 500 / 1000, that is flat; peak RSS grows 1.49 GB to 5.91 GB (about 4.0x for 8x signatures) while proof size grows 188 to 229 KiB (+21.8%). This is independent, third-party, 2026 corroboration that only proof bytes amortize.

We intend to reuse their schema conventions rather than reinvent them: coarse machine fingerprint as grouping key with filename `<ISO8601>__<fingerprint>.json`; raw `samples_ns` retained alongside derived `timing{n, mean, stddev, min, p5, p50, p95, max}`; `resources{cpu_percent, rss_bytes, n_samples, interval_ms}`; and their disclosure that "CPU percentage is summed across logical cores". Their file format has **no scheme axis and no prover axis** (`toolchain.git_shas` is hardcoded to leansig and leanmultisig), so our results are a superset, not a drop-in contribution to their dataset.

### 3.5 ZK-ACE: a Circle STARK batch table that measures something else

Wang, "ZK-ACE: Identity-Centric Zero-Knowledge Authorization for Post-Quantum Blockchain Systems", [arXiv:2603.07974](https://arxiv.org/abs/2603.07974), v1 9 Mar 2026, v3 28 May 2026 (accessed 2026-07-23).

Table 7, verbatim, caption "Circle STARK batch prove/verify timings and proof sizes (single-threaded, Apple Silicon, Criterion.rs medians, zk-ace v0.4.0)":

| N | log | Prove | Prove/tx | Verify | Verify/tx | Proof | Proof/tx |
|---|---|---|---|---|---|---|---|
| 1 | 9 | 14.29 ms | 14.29 ms | 3.18 ms | 3.18 ms | 122 KB | 122 KB |
| 2 | 10 | 28.89 ms | 14.45 ms | 5.27 ms | 2.64 ms | 139 KB | 69 KB |
| 4 | 11 | 57.84 ms | 14.46 ms | 9.83 ms | 2.46 ms | 156 KB | 39 KB |
| 8 | 12 | 126.10 ms | 15.76 ms | 19.90 ms | 2.49 ms | 185 KB | 23 KB |
| 16 | 13 | 236.39 ms | 14.77 ms | 37.97 ms | 2.37 ms | 206 KB | 13 KB |

This is the closest published analogue to our headline chart, and it independently reproduces the shape: prove per transaction flat, proof bytes amortizing 122 KB to 13 KB.

**Three qualifications a reviewer will otherwise raise against us.**

- **It does not verify a PQ signature in-circuit at all.** Its thesis is avoiding that: it "replaces transaction-carried signature objects with identity-bound ZK statements", with a "deterministic identity derivation primitive (DIDP)" treated "as a black box". Section 12.2 concedes "Verifying them inside ZK circuits merely relocates the cost via expensive lattice arithmetic in prover circuits". Its 14.5 ms is the cost of a Poseidon2 / identity-recovery circuit. A reader who only sees Table 7 will assume it is the same measurement it is not.
- **Hardware is weaker than usually reported.** Tables 5, 6 and 7 say only "single-threaded, Apple Silicon"; section 12.2 says "Apple Silicon (M-series)". "Apple M3 Pro" appears **only** in the Table 12 caption. Attributing Table 7 to an M3 Pro is an inference, not what the paper states.
- **The commonly quoted range "14.29 to 14.77 ms/tx" is wrong.** Prove/tx is not monotone: 14.29, 14.45, 14.46, **15.76** (N=8), 14.77 (N=16). The true range is 14.29 to 15.76 ms/tx and the peak is at N=8, unexplained by the paper. N takes five values (1, 2, 4, 8, 16), not sixteen.

The authors flag their own artifact gap, verbatim: these figures "should be interpreted as reference measurements for the reported implementation and hardware; a public artifact should identify the repository URL, commit hash, Rust toolchain, benchmark commands, and machine configuration used to reproduce them." Treat Table 7 as published-but-unreproduced.

## 4. Native PQ benchmark baselines we cross-check against

### 4.1 The NIST PQC signature zoo, and why it is only an order-of-magnitude check

[NIST PQC Signature Zoo](https://pqshield.github.io/nist-sigs-zoo/), Thom Wiggers / PQShield; data "last updated 2026-04-03"; repo [PQShield/nist-sigs-zoo](https://github.com/PQShield/nist-sigs-zoo) HEAD `c414d7d` 2026-07-22; CC BY-SA 4.0 (accessed 2026-07-23). 116 parameter sets.

Measured-and-published rows we care about (verbatim from `data/schemes/*.yaml` at `c414d7d`; sizes in bytes, cycles, and microseconds):

| scheme | pk | sig | sign cycles | verify cycles | sign us | verify us | `perf_source` |
|---|---|---|---|---|---|---|---|
| ML-DSA-44 | 1312 | 2420 | 172926 | 76028 | 61.9 | 27.2 | OQS bench |
| ML-DSA-65 | 1952 | 3309 | 293694 | 124851 | 107.5 | 46.1 | OQS bench |
| ML-DSA-87 | 2592 | 4627 | 368956 | 197511 | 136.0 | 73.0 | OQS bench |
| Falcon-512 | 897 | 666 | 623227 | 65140 | 224.4 | 23.3 | OQS bench |
| Falcon-1024 | 1793 | 1280 | 1253155 | 124210 | 448.2 | 44.9 | OQS bench |
| SLH-DSA-SHAKE-128s | 32 | 7856 | 2781232192 | 2682410 | 955962.8 | 912.7 | submission document |
| SLH-DSA-SHAKE-128f | 32 | 17088 | 132061616 | 7784088 | 43140.7 | 2486.9 | submission document |
| Ed25519 | 32 | 64 | 126662 | 334852 | 42.0 | 110.6 | OpenSSL bench |
| ECDSA P-256 | 65 | 72 | 79393 | 235386 | 36.3 | 107.2 | *(no `perf_source` field)* |

**Four corrections to how this source was handed to us, one of them disqualifying for its intended use.**

1. **There are no keygen numbers.** The signature dataset has no keygen column and no keygen field. Keygen exists only on the separate KEMs page.
2. **It is not one measurement campaign.** The site discloses a benchmark environment (`data/benchmark_env.yaml`, 2026-06-26: "12th Gen Intel(R) Core(TM) i7-12650H", Debian 13, "Median over 1000 iterations", "rdpmc CPU_CYCLES", thread pinned) but our six target rows are attributed to four different provenances, as the table shows. **Our inference** confirms the heterogeneity: dividing each row's cycles by its microseconds implies clocks of 2794, 2777, 3061, 3016, 2187 and 3492 MHz. Those rows cannot have come from one pinned machine.
3. **It is x86 only.** No aarch64, no Apple Silicon anywhere in the dataset. It cannot validate anything we measure on Apple Silicon.
4. **secp256k1 is absent.** `ECDSA.yaml` has only P-256, P-384, P-521. The Bitcoin and Ethereum curve contributes nothing here.

Disconfirmations found while checking it, all flagged:

- `https://pqshield.github.io/nist-sigs-zoo/wide.html` **returns HTTP 404** while still being surfaced by search engines. Do not cite it. The citable artifacts are the repo YAMLs.
- **Two credible, published ML-DSA-44 figures differ by about 1.9x.** The zoo says 172,926 sign cycles; [pq-crystals' own Dilithium page](https://pq-crystals.org/dilithium/index.shtml) says "Intel Core-i7 6600U (Skylake)" AVX2 gen 124031, sign 333013, verify 118412 (accessed 2026-07-23). The zoo's own stale `static/data/parametersets.csv` still carries the pq-crystals values verbatim. If our harness lands anywhere in 170k to 340k cycles for ML-DSA-44 sign, that is not evidence our harness is broken.
- **The Ed25519 and ECDSA P-256 rows are slow and will cause a false alarm against our harness.** Ed25519 sign at 126,662 cycles is roughly 3x a well-optimized implementation; ECDSA P-256 verify at 235,386 cycles is several times slower than an OpenSSL build with nistz256 assembly. If our Rust harness reports Ed25519 sign 2 to 3x faster than this table, our harness is probably right. Do not "correct" toward these numbers.
- **Internal inconsistency in the repo.** `benchmark_env.yaml` lists submodule sources for mldsa, slhdsa and fndsa, implying those were measured on the i7-12650H, while the scheme YAMLs attribute them to "OQS bench" and "submission document". One of the two is stale; we could not resolve which is authoritative.
- The legacy `static/data/parametersets.csv` Ed25519 row is incoherent (`42000` cycles paired with `0.00274` ms implies a 15 GHz CPU). Use `data/schemes/*.yaml` only.

The maintainer's own caveat is worth quoting: commit `c414d7d` (2026-07-22) adds an eBACS footer link with the message that it "Points to bench.cr.yp.to for more comprehensive benchmarks across more platforms than this site's own KEM/signature timings".

**Net rule: cite the zoo for sizes and for order-of-magnitude ranking. Do not cite it as a precision timing baseline, do not cite it for keygen, and do not use it to validate anything measured on Apple Silicon.**

### 4.2 eBACS / SUPERCOP, and the aarch64 gap

[bench.cr.yp.to/results-sign.html](https://bench.cr.yp.to/results-sign.html), page version "20260717 22:08:42" (accessed 2026-07-23). It has aarch64 machines, verbatim: "2023 Broadcom BCM2712; 4 x 1500MHz; pi5", "2019 Broadcom BCM2711; 4 x 1500MHz; pi4b", "2018 Broadcom BCM2837B0; 4 x 1400MHz; pi3aplus". **No Apple Silicon.** No secp256k1 ECDSA. And its `dilithium2` / `falcon512dyn` / `falcon512tree` entries are round-3 submissions, **not** the final FIPS 204 / FIPS 206 parameterizations, so they are not interchangeable with ML-DSA-44.

### 4.3 What is genuinely missing: an aarch64 baseline

**We found no citable absolute secp256k1 verify figure on aarch64 anywhere.** The [delvingbitcoin thread 2087](https://delvingbitcoin.org/t/libsecp256k1-vs-openssl/2087) (Pieter Wuille, 2025-11-02) gives "I ran your script on a Ryzen 5950X CPU, with 500000 iterations...showing a speed ratio of 8.5 currently" and quotes the original 2015 claim of "anywhere between 2.5 and 5.5 times faster" than OpenSSL; its arm64 run publishes only a relative bar chart with no CPU model and no units (accessed 2026-07-23). Web-search snippets floating "47.6 microseconds" and "88.70 microseconds" for libsecp256k1 verify trace to aggregator pages we did **not** fetch. **We explicitly do not assert them.**

Consequence: we must publish our own native aarch64 baselines for every scheme in the benchmark, framed as "no prior aarch64 baseline was citable", never as "nobody has benchmarked these".

Note on pqm4: it targets Cortex-M4 only (32-bit embedded). Its numbers must never share an axis with aarch64 or x86 numbers.

Unresolved: we could not extract the timing tables from [arXiv:2601.17785](https://arxiv.org/abs/2601.17785), "Performance Analysis of Quantum-Secure Digital Signature Algorithms in Blockchain" (2026-01-27). The PDF's compressed streams defeated text extraction and no hardware description was recoverable. **Treat as unverified; do not cite its numbers.**

## 5. Naive-on-chain comparison points

### 5.1 Lattice: a PQ-only L1 that posts ML-DSA-44 signatures raw

Trejo Pizzo, "Lattice: A Post-Quantum Settlement Layer", [arXiv:2603.07947](https://arxiv.org/abs/2603.07947), v1 9 Mar 2026, "Version 0.8.0", primary category quant-ph, cross-list cs.CR (accessed 2026-07-23).

A suspected arXiv ID transposition against ZK-ACE (2603.07974) was checked and is a **false alarm**: both IDs resolve to distinct real papers submitted the same day. Do not "correct" either.

The paper enforces "ML-DSA-44 as the sole signature algorithm from the genesis block. ECDSA is disabled at the consensus level", with **no ZK or STARK layer** (a grep for STARK / zkVM / zero-knowledge finds only a Limitations line: "Lattice does not implement ring signatures, zero-knowledge proofs"). That makes it the clean left-hand bar for "what does naive on-chain PQ cost".

Verbatim size model, section 7.1.1, Bitcoin/ECDSA vs Lattice/ML-DSA-44: "Public key 33 bytes | 1,312 bytes"; "Signature 72 bytes | 2,420 bytes"; "P2PKH scriptSig ~107 bytes | ~3,740 bytes"; "Typical tx (1-in, 2-out) ~250 bytes | ~4,000 bytes"; "Effective tx/block ~2,800 | ~2,400". Throughput: "TPS = floor(56,000,000/16,000) / 240s = 3,500/240 ~= 14.6 tx/s (theoretical max)", rising to "~47 tx/s" post-SegWit, "~3.6 tx/s" at 25% utilization. Storage: "G_max_year ~= 7.4 TB/year".

Native benchmarks, "Benchmarks on Intel i7-12700 (single core)", ML-DSA-44 | Falcon-512 | ECDSA: keygen "~0.05 ms | ~8.0 ms | ~0.04 ms", signing "~0.15 ms | ~0.40 ms | ~0.05 ms", verification "~0.05 ms | ~0.05 ms | ~0.15 ms".

**Do not use this paper as an ECDSA baseline.** It contradicts itself. Section 3.2 gives ECDSA verify "~0.15 ms" and concludes "ML-DSA-44 verification is actually faster than ECDSA verification"; section 9 gives "ECDSA's ~40,000-70,000 signatures/second" (0.014 to 0.025 ms) and says ML-DSA-44 is "approximately 2-3x slower"; section 3.4.3 gives a third pair, "~40K/s | ~20K/s (2x slower)". The two sections reach opposite conclusions about which verifies faster, and ZK-ACE independently reports secp256k1 ECDSA verification at "approximately 50--100 us", consistent with section 9 and not with section 3.2.

**Also disconfirmed:** the paper's Installation section lists `git clone https://github.com/lattice-network/lattice.git`. That repository **returns HTTP 404** as of 2026-07-23. No code exists at the URL the paper publishes, so every figure in it is author-claimed and unreproducible. It is a single-author v1 preprint with no journal reference and no peer review.

### 5.2 OpenZeppelin cairo-pq-verifiers: the conformance-versus-cost axis, quantified on-chain

[OpenZeppelin/cairo-pq-verifiers](https://github.com/OpenZeppelin/cairo-pq-verifiers), MIT; repo created 2026-06-24, last commit [`bece5c0`](https://github.com/OpenZeppelin/cairo-pq-verifiers/commit/bece5c07eeea5784e570c01108ec000d2d04ae40) 2026-07-22; `results/results.json` `"generated": "2026-07-14 16:41"` (accessed 2026-07-23).

**This publishes Cairo steps and Starknet L2 gas only. No proving time, no prover, no hardware, no RAM, no proof size.** We verified that three ways: its "What we measure" section, its "Method" section ("Numbers come from Starknet Foundry (gas, steps, builtins), a release build (class size), and cairo-profiler"), and a grep of the full 32,777-byte `results.json` for any key matching `prov|time|second|ms|sec`, which returns zero matches. Any chart of ours placing 97,681 next to a prover-second number would be a category error.

Falcon-512 bare verify, measured-and-published (L2 gas / Cairo steps):

| variant | L2 gas | steps | conformant? |
|---|---|---|---|
| ECDSA-STARK (classical control) | 30,855 | 152 | classical baseline |
| Falcon-512 Poseidon (native) | 12,045,449 | 97,681 | no, non-standard hash-to-point |
| Falcon-512 hint (BLAKE2s) | 12,809,640 | 104,854 | no, non-standard |
| Falcon-512 direct (BLAKE2s) | 23,005,760 | 205,781 | no, non-standard |
| Falcon-512 SHAKE-256 (standard) | 50,163,668 | 312,892 | yes |
| Falcon-512 SHAKE-256 direct (standard) | 60,359,788 | 413,819 | yes, closest to a bare FIPS signature |

The repository states the conformance penalty itself, so this is not an inference we are imposing on them: `schemes.json` says the SHAKE-256 bare verifier "uses 4.16x the gas and 3.20x the steps of the native-Poseidon variant". Note that 3.2x applies to the SHAKE **hint** variant; the variant OpenZeppelin itself calls "the closest on-chain match to a bare standard Falcon signature" is `falcon_512_shake_direct` at 413,819 steps, that is **4.24x** Poseidon. Quoting 3.2x understates the penalty for the most faithful variant.

**ML-DSA-44 here is an explicit stub, and that is the strongest single piece of evidence for our empty cell.** `crates/ml_dsa_44/src/lib.cairo` opens verbatim with "//! STUB" followed by "placeholder for an ML-DSA-44 (CRYSTALS-Dilithium, FIPS 204) verifier. //! NOT a real implementation." The body is `public_key.len() == 43 && signature.len() == 79`, ignoring `message_hash` entirely and returning true for any input of the right shape. Its README says "**Status:** stub (pending implementation)" and "No measurements yet". A top-tier auditor has an ML-DSA-44 crate with a NIST reference and a planned encoding, and still has zero measurements for it.

Two fairness points we impose on ourselves. Falcon is **FIPS 206 draft**, not final, so "standards-conformant Falcon" means Falcon-spec / falcon.py-interoperable, never "FIPS-conformant" unqualified. And OpenZeppelin does not hide the relaxation: it labels the non-standard variants, ships two conformant ones, and quantifies the gap. Our contribution cannot be "we caught someone relaxing the standard"; it is that conformance-versus-cost is quantified **on-chain** and not quantified in **prover** terms by anyone.

Process warning, recorded because it nearly cost us the finding: a page-summarising fetch of this repository returned a confident, well-formatted four-row table that silently dropped both `direct` variants, including the 413,819-step row, and mis-described the variant axis. All figures above came from `raw.githubusercontent.com` and the GitHub API.

## 6. Competing approaches, and why a reader should care

A reader deciding how to put PQ signatures behind a settlement layer has at least four options. A zkVM benchmark that pretends it is the only one is not credible.

**a. Verify the signature inside a general-purpose zkVM.** RISC Zero, SP1, Cairo/Stwo. The lane PQ-STARK-BENCH measures. Advantage: reuse of an existing prover and a Rust or Cairo implementation of the scheme. Disadvantage: no lattice or NTT precompile exists in any of them (verified in section 7), so the arithmetic runs as interpreted RISC-V.

**b. Write a custom AIR for the scheme.** [starkware-bitcoin/falcon-air](https://github.com/starkware-bitcoin/falcon-air) is a hand-written Stwo AIR for Falcon's Z_q arithmetic (q = 12289) with NTT and INTT circuits, created 2025-07-17, last updated 2025-11-27, 2 stars (accessed 2026-07-23). Its README self-describes as "Research prototype. This code has not been audited." It publishes **no performance numbers** and has **no LICENSE file** (its own README says "If you intend this project to be MIT-licensed, add a LICENSE file"), so it is legally unsafe to vendor. Its existence means the honest framing of our benchmark is "general-purpose zkVM lane versus custom-AIR lane", and we should say so rather than imply the zkVM route is the only one.

**c. Lattice-native aggregation, no zkVM.** Aardal et al. published compact aggregation of Falcon signatures via LaBRADOR at CRYPTO 2024, that is **later** than BTQ 2023 and a lattice SNARK rather than a STARK or zkVM. It does not threaten the chronology in section 2, but it is the strongest adjacent Falcon-in-proof-system result and is named by s2morrow itself as their intended comparison baseline ("benchmark vs other approaches (e.g. LaBRADOR)"). Anyone reading our benchmark will ask "why not LaBRADOR"; the answer is that it is a different trust and tooling model, not that it is worse.

**d. Hash-based aggregation, either in a bespoke AIR or in a zkVM.** ePrint 2021/1048 (bespoke AIR, section 2.1), HAPPIER (RISC Zero, section 2.3), leanSig/leanVM (section 3.4), and s2morrow's SPHINCS+-128s lane (section 3.1). This family is consistently cheaper in-circuit because the schemes can be instantiated over an arithmetization-friendly hash. That is exactly why it does not answer the FIPS question: ML-DSA and SLH-DSA cannot swap their hash.

**e. Avoid verifying a signature in the proof at all.** ZK-ACE (section 3.5) replaces the signature object with an identity-bound ZK statement and explicitly declines to measure in-circuit PQ verification, calling that alternative "roughly 500--2,300x" more expensive by its own estimate. Useful to cite as the strongest argument against our whole premise, and honest to include.

Also on the classical side, [EIP-7619 "Precompile Falcon512 generic verifier"](https://eips.ethereum.org/EIPS/eip-7619) exists as L1 prior art on making Falcon verification cheap by protocol change rather than by proving. We surfaced it in search but did **not** fetch it; it is listed here as a lead, not as a verified citation.

## 7. Ecosystem context, stated precisely

### 7.1 Starknet

StarkWare, "The Architecture Advantage: Starknet's Quantum Readiness Roadmap", published **2026-06-30** ([starkware.co](https://starkware.co/blog/the-architecture-advantage-starknets-quantum-readiness-roadmap/), accessed 2026-07-23). This is more specific and more recent than "StarkWare announced a roadmap" suggests: three phases with per-item estimates.

- Phase 1, "Securing All New Activity": "substituting Pedersen hashing, which inherits elliptic-curve assumptions, for BLAKE2, which does not", across state, chain environment and consensus. Estimate verbatim: "OS config hash live in early July; other items - approximately two months from start." Also: "all new features will require quantum-safe implementations ... including ... consensus signatures onchain using PQ signature schemes, such as Falcon-512."
- Phase 2, legacy contract migration toolkit: "approximately one month after Phase 1 for easy path on StarkWare side; TBD on dapps side."
- Phase 3, external dependencies: "The Cairo VM's secp256k1/r1 syscalls, which support L1-to-L2 messaging and the Ethereum bridge, are quantum-vulnerable", estimate "1 month post-Ethereum migration"; plus blob DA "anchored by a KZG commitment, which is quantum-vulnerable".

**It is not a commitment.** Quote the hedge if you characterise it as one: "All protocol-level changes to the Starknet network are subject to approval through Starknet's governance process. No timeline or outcome described herein is guaranteed."

What Starknet accounts use today: the protocol mandates nothing. [Starknet docs](https://docs.starknet.io/learn/protocol/cryptography) state "The STARK curve is commonly used in smart contracts, but is not distinguished by the Starknet protocol" (accessed 2026-07-23), and [OpenZeppelin Contracts for Cairo 3.x](https://docs.openzeppelin.com/contracts-cairo/3.x/accounts) says "usually most account implementations validate transactions using the Stark curve which is the most efficient way" (accessed 2026-07-23). So the de-facto default is ECDSA over the 251-bit STARK curve: STARK-friendly, and broken by Shor. PQ accounts are deployable today; StarkWare's own post says "Starknet users can deploy a PQ wallet today, which S2morrow has already demonstrated".

Reachability note for anyone reproducing this: `https://docs.starknet.io/learn/protocol/accounts/introduction` returns HTTP 404; the working path is `https://docs.starknet.io/learn/protocol/accounts.md`. The site publishes `.md` variants of every page and an `llms.txt` index; the HTML renders math as unreadable duplicated LaTeX.

### 7.2 Ethereum

EF blog, "lean Ethereum", Justin Drake, 31 July 2025 ([blog.ethereum.org](https://blog.ethereum.org/2025/07/31/lean-ethereum), accessed 2026-07-23): "Hash-based cryptography is emerging as the ideal foundation for lean Ethereum", with "CL: hash-based aggregate signatures upgrade BLS signatures / DL: hash-based DAS commitments upgrade KZG commitments / EL: hash-based real-time zkVMs upgrade EVM re-execution". The post is explicitly labelled "a Drake take".

[ethereum.org quantum-resistance page](https://ethereum.org/roadmap/future-proofing/quantum-resistance/), "Page last update: April 9, 2026" (accessed 2026-07-23): "a structured 'Lean Ethereum' roadmap targeting 2029 for full post-quantum protection"; "Ethereum will replace BLS signatures with leanXMSS"; "Rather than a single protocol-wide migration, Ethereum plans to use account abstraction (specifically EIP-8141, being considered for Hegota in second half of 2026) to give users signature agility"; and on proofs, "STARKs, which rely on hash functions rather than elliptic curves, are already quantum-resistant". Threat timing quoted there: "In March 2026, Google Quantum AI published research estimating that breaking 256-bit elliptic curve cryptography ... could require roughly 1,200 logical qubits", and "NIST anticipates deprecating ECDSA by 2030 and disallowing it by 2035".

### 7.3 The motivating gap, stated so it cannot be refuted in one sentence

The sloppy version, "settlement is post-quantum but account signatures are not", is attackable three ways and we do not use it.

- StarkWare's own roadmap concedes non-PQ pieces **inside** settlement: Pedersen hashing "inherits elliptic-curve assumptions" and was still in state commitment and address derivation until Phase 1; the L1 bridge uses quantum-vulnerable secp syscalls; blob DA rests on KZG.
- On Starknet the account scheme is not protocol-fixed, and Falcon-512 accounts are deployable today. Ethereum is heading the same way via EIP-8141. This is a defaults-and-deployed-base gap, not a protocol-impossibility gap.
- StarkWare's own marketing sentence "Starknet's architecture does not rely on quantum-vulnerable cryptography at the account or settlement layers" is in tension with its own Phase 1 and Phase 3. Do not quote it as evidence for our gap; it argues against having one.

The formulation we use: *the proof system securing settlement is already hash-based and post-quantum, while the signatures authorizing the transactions inside it are elliptic-curve on essentially all deployed accounts; both StarkWare and the Ethereum Foundation now plan to close that at the account layer via signature agility rather than a protocol-wide swap, which makes the prover cost of verifying PQ signatures inside the proof the binding constraint.*

### 7.4 Precompile asymmetry, on all three platforms

This is the single disclosure that decides whether our charts are honest.

**SP1.** The full syscall enum at tag v6.3.1 ([`crates/core/executor/src/syscall_code.rs`](https://raw.githubusercontent.com/succinctlabs/sp1/refs/tags/v6.3.1/crates/core/executor/src/syscall_code.rs), accessed 2026-07-23) contains `SHA_EXTEND`, `SHA_COMPRESS`, `KECCAK_PERMUTE`, `ED_ADD`, `ED_DECOMPRESS`, `SECP256K1_*`, `SECP256R1_*`, `BN254_*`, `BLS12381_*`, `UINT256_MUL`, `POSEIDON2`. **There is no lattice, NTT, ML-DSA, Falcon or SHAKE-specific precompile.** Round-1's phrasing "SP1 has a sha3/SHAKE precompile" should be tightened: SP1 patches the `sha3` crate so SHA3 and SHAKE route through `KECCAK_PERMUTE`; there is no SHAKE circuit.

**RISC Zero.** **Correction to a round-1 finding.** We were told RISC Zero "does not list" a SHAKE path. That is half wrong. The [precompiles page](https://dev.risczero.com/api/zkvm/precompiles) (accessed 2026-07-23) patches `tiny-keccak` 2.0.2 (gated behind the `unstable` feature), and `tiny-keccak` provides Keccak-f[1600] plus Shake128 and Shake256. The v3.0.1 release notes state "Stabilize bigint and keccak features". So a RISC Zero ML-DSA guest **can** get an accelerated SHAKE path. A chart claiming "RISC Zero has no SHAKE acceleration" would be wrong. The asymmetry that actually stands is the **lattice/NTT** one: neither prover has an NTT or ML-DSA precompile, while both accelerate Ed25519 and ECDSA. (Note: `https://dev.risczero.com/api/zkvm/acceleration` fetched with HTTP 200 but an **empty body** on 2026-07-23; use `/precompiles`.)

**Starknet.** OpenZeppelin names SNIP-32 (a native `keccak_f1600` syscall) twice as the thing that "would close the remaining gap" for standards-conformant Falcon. Same story, third platform. We did not verify SNIP-32's status.

**Cross-cutting distortion, in plain sight.** The feltroidprime s2morrow fork swapped SHAKE-256 for Poseidon in hash-to-point precisely because the native hash is cheap in its proof system. OpenZeppelin measured that same swap at 4.16x gas and 3.20x steps. This is the same class of distortion as accelerating Ed25519 but not ML-DSA. Every chart must disclose the hash primitive and the precompile availability, or a reviewer will correctly call the comparison rigged.

### 7.5 Two operational facts that constrain what we can even measure

**Bonsai pricing does not exist any more.** `https://dev.risczero.com/api/bonsai/bonsai-overview` **returns HTTP 404** (accessed 2026-07-23), and Boundless docs state verbatim "Bonsai was RISC Zero's centralized proving service, delivering proofs via an API. As of December 2025, Bonsai is no longer available." ([docs.boundless.network](https://docs.boundless.network/developers/tutorials/bonsai), accessed 2026-07-23). Any work citing Bonsai dollars-per-proof is citing a dead product. Boundless is a bid-based market with no published fixed rate. On the SP1 side, the [prover network FAQ](https://docs.succinct.xyz/docs/sp1/prover-network/faq) says "The price per PGU is set through a competitive auction" (accessed 2026-07-23). **Cost per proof is not published on either network.** We therefore report prover seconds and peak RSS on named hardware as the primary metric, which is also what ePrint 2021/1048 did.

**Cycles are not a shared unit across provers, and the ISA differs.** SP1 v6's guest target is `riscv64im-succinct-zkvm-elf` (`DEFAULT_TARGET` in [`crates/build/src/lib.rs`](https://raw.githubusercontent.com/succinctlabs/sp1/refs/tags/v6.3.1/crates/build/src/lib.rs), accessed 2026-07-23; the `zkevm/README.md` at the same tag says "RV64IM, LP64, soft-float"), while RISC Zero is RV32IM. SP1 charges a flat +256 clock per syscall regardless of precompile; RISC Zero charges paging, "A page-in or page-out operation takes between 1094 and 5130 cycles; 1130 cycles on average" ([Guest Optimization Guide](https://dev.risczero.com/api/zkvm/optimization), accessed 2026-07-23). Succinct's own docs disown cycles as a proving-cost proxy and ship "prover gas" instead. **Cycle counts from the two systems must never share an axis.**

Note for anyone rechecking: the SP1 ["Compiling"](https://docs.succinct.xyz/docs/sp1/writing-programs/compiling) doc page still says `riscv32im-succinct-zkvm-elf`, contradicting both the install page and the v6.3.1 source. That page is stale. Any prior work resting on "SP1 is riscv32im" needs rechecking against v6.x.

## 8. Honest positioning: what is new here, and what is reproduction

### 8.1 What we explicitly do not claim

- Not the first PQ signature verified in a STARK. ePrint 2021/1048 did that in 2021 with published benchmarks.
- Not the first Falcon in Cairo or on Starknet. BTQ 2023, feltroidprime and OpenZeppelin all precede us.
- Not the first prover-cost-versus-batch-size curve for a PQ signature in a zkVM. HAPPIER (RISC Zero, 2025) and leanBench (leanVM, 2026) both publish one.
- Not a discovery that amortization is flat. That is stated outright in the Winterfell README, visible in ePrint 2021/1048 Table 3, reproduced by ZK-ACE Table 7 and by leanBench. Anyone pitching it as a finding gets embarrassed.
- Not a claim that anyone is hiding the conformance penalty. OpenZeppelin quantifies it themselves.

### 8.2 What is genuinely new, scoped exactly

1. **Measured prover cost for a FIPS signature scheme in a general-purpose zkVM.** Every prior in-zkVM measurement we found uses a hash-based scheme instantiated over an arithmetization-friendly hash chosen for the proof system (Lamport+ over Rescue-Prime, generalized XMSS over SHA or Poseidon, leanSig over Poseidon). ML-DSA-44 in a general-purpose zkVM is measured nowhere: the OpenZeppelin ML-DSA-44 crate is a stub that returns true for any well-shaped input, the sp1-ntt-gadget is an NTT gadget over an empty repository whose cycle arithmetic does not close, and s2morrow's proving-benchmarks checkbox is unchecked and its sweep hardcoded to `--num_signatures 1`.
2. **Prover cost for a standards-conformant Falcon path.** OpenZeppelin quantified conformance-versus-cost in Cairo steps and L2 gas. Nobody has quantified it in prover seconds or prover RAM, on any prover.
3. **More than one prover, on the same schemes, with the backend asymmetry disclosed.** HAPPIER is RISC Zero only. leanBench is leanVM only. ZK-ACE is Circle STARK only. s2morrow is Stwo only and unmeasured.
4. **Peak RSS on named hardware, alongside wall clock.** HAPPIER publishes neither RAM nor hardware while attacking prior work on RAM. s2morrow publishes neither. Dilithium-ZK publishes neither.
5. **Wall clock for real batches, not an extrapolated critical path.** HAPPIER's per-N totals come from `mean(leaf) + merge * log2(N)/log2(arity)` over runs that aggregate two real signatures.
6. **Native aarch64 baselines for all schemes,** because none is citable: the signature zoo is x86 only, eBACS aarch64 is Raspberry Pi class only and its lattice entries are round-3 not FIPS-final, and no absolute secp256k1 aarch64 verify figure was findable at all.
7. **A precompile and hash-primitive disclosure on every chart,** as a methodological contribution rather than a number: the classical baselines are precompile-accelerated on both provers while the lattice schemes are not, and correcting the round-1 error about RISC Zero's `tiny-keccak` path means we can and should publish ML-DSA-44 on RISC Zero **both ways**, stock `sha3` versus patched `tiny-keccak`. That delta appears to be unmeasured by anyone.

### 8.3 What is deliberate reproduction

- Re-running HAPPIER on current RISC Zero (it is pinned to r0vm 1.2.5, April 2025) with real batches and RSS instrumentation, to supply the memory row it omitted and to check its "2-3 MB" claim against the 7 MB to 30 MB range visible in its own logs.
- Re-measuring ePrint 2021/1048's batch axis on modern hardware, as a sanity anchor against a well-specified 2021 result, and to check the Winterfell README table against the paper's Table 3 where the two disagree.
- Re-deriving the amortization shape from our own data, presented as confirmation of known prior art, not as a finding.

### 8.4 Known limitations of this survey

- The HAPPIER chapter is paywalled and unread. Whether it contains a memory column, hardware spec, or non-extrapolated timing is unresolved.
- arXiv 2601.17785's tables could not be extracted.
- We did not fetch EIP-7619, the OpenZeppelin `pq-accounts/USAGE.md` devnet receipt-gas data, Vitalik's February 2026 PQ roadmap post, the leanroadmap phase claims, or the Aardal et al. LaBRADOR paper directly. Those are listed as leads, not citations.
- SNIP-32's status on Starknet is unverified.
- Nothing on X/Twitter was fetchable. A post attributed to @Starknet ("Post-quantum wallets are now live on Starknet") appeared in search results only and is **not** cited as evidence; the same substance is available from starkware.co and s2morrow.xyz, which we did fetch.

## 9. Full prior-work table

All links accessed 2026-07-23. "In-circuit?" means the PQ signature is verified inside the proof system. "Numbers?" means prover-cost figures specifically, not sizes or gas.

| Work | Date | Scheme(s) | Prover / system | In-circuit? | Numbers published? | Reproducible? | Link |
|---|---|---|---|---|---|---|---|
| Khaburzaniya et al., ePrint 2021/1048 (AsiaCCS 2022) | 2021-08-16, rev. 2022-03-14 | Lamport+ (w=2 WOTS) over Rescue-Prime, not FIPS | Bespoke AIR, Winterfell-class STARK | Yes | Yes: prover time, prover RAM, sig size at n=128..1024, two soundness levels, named 8-core i9 hardware. No amortized figure | Partly: its own artifact URL 404s; Winterfell attribution is our inference and the two tables disagree | [ePrint](https://eprint.iacr.org/2021/1048) · [Wayback PDF](https://web.archive.org/web/20260121180642if_/https://eprint.iacr.org/2021/1048.pdf) |
| BTQ + StarkWare, Falcon in Starkware | **2023-09-18** (not 2024) | Falcon, parameter set not published | Cairo; proof generation not stated | Claimed, not evidenced | **No.** No time, steps, constraints, RAM, proof size or gas | No code, no artifact | [btq.com](https://www.btq.com/blog/completing-the-first-falcon-signature-verification-in-starkware-initiating-the-transition-to-a-quantum-safe-ethereum) |
| HAPPIER, LightSec 2025 | Conf. 2025-09-01; online 2026-01-31 | Generalized XMSS (b-wagn hash-sig), not RFC 8391 XMSS | RISC Zero r0vm 1.2.5 | Yes | Prover time vs N=128..262144, arity 2/4/8, but **extrapolated** from 2-signature runs. **No RAM, no hardware** | Code and raw logs public; paper paywalled and unread | [DOI](https://link.springer.com/chapter/10.1007/978-3-032-15541-2_1) · [code](https://github.com/ArdaSaygan/PQ-Aggregation-via-Recursive-SNARKs) |
| starkware-bitcoin/s2morrow | created 2025-04-27; frozen since 2026-03-23 | Falcon-512, SPHINCS+-128s (SHA2 and Blake2s) | Cairo / Stwo | Yes, verifiers implemented | **No.** `- [ ] Stwo proving benchmarks` unchecked; sweep hardcoded `--num_signatures 1` | Verifier code MIT and vendorable; proving pipeline blocked on a private SSH dependency | [repo](https://github.com/starkware-bitcoin/s2morrow) |
| feltroidprime/s2morrow + s2morrow.xyz | fork, last commit 2026-03-05 | Falcon-512 only, **Poseidon** hash-to-point, not FIPS | Cairo, on-chain verification | Yes, but single signature | Steps and L2 gas only, no prover cost. Its own two `verify` gas figures disagree (~13.2M vs ~9.5M) | Code public, MIT | [repo](https://github.com/feltroidprime/s2morrow) · [site](https://www.s2morrow.xyz/) |
| OpenZeppelin cairo-pq-verifiers | created 2026-06-24; data 2026-07-14 | Falcon-512 x5 variants; ML-DSA-44 and Poseidon-WOTS+ **stubs** | Cairo VM execution cost, no prover | On-chain verify, not proving | Cairo steps and L2 gas only. **No prover time, hardware, RAM or proof size** | Yes, MIT, CI-ratcheted, falcon.py fixtures | [repo](https://github.com/OpenZeppelin/cairo-pq-verifiers) |
| leanEthereum/leanBench | created 2026-05-06; last 2026-06-20 | leanSig (generalized XMSS over Poseidon), not FIPS | leanVM only (not six provers) | Yes | Yes: mean_ns, peak RSS, proof KiB, flat 125..1000 and tree fan-in 2/4/8, 101 runs, 7 machines | Yes, MIT, seed-pinned `0xC0FFEE`; dashboard needs JS | [repo](https://github.com/leanEthereum/leanBench) |
| ZK-ACE, arXiv 2603.07974 | v1 2026-03-09, v3 2026-05-28 | None in-circuit; identity statement replaces the signature | Circle STARK (Stwo) | **No, by design** | Table 7: prove/tx 14.29..15.76 ms, proof/tx 122..13 KB, N=1,2,4,8,16, "Apple Silicon (M-series)" | Authors state no reproducible artifact accompanies the figures | [arXiv](https://arxiv.org/abs/2603.07974) |
| Dilithium-ZK / `sp1-ntt-gadget` | 2025-12-08 | ML-DSA-**65**, NTT gadget only, "not a complete ML-DSA signature verifier" | SP1 v5.x, SP1 Network (outsourced) | Partial, gadget only | 22.07 s / 260 B / ~$0.045 vendor-claimed. **Hardware not published.** Cycle breakdown sums to 1.1M against a stated total of 5,625,411 | **No.** Linked repo empty (0 bytes, no commits); manifest excludes tests, examples, scripts | [Medium](https://medium.com/@phillyj1026/building-a-zero-knowledge-verifier-for-dilithium-signatures-ntt-gadget-implementation-in-sp1-6c50ab262836) · [crate](https://crates.io/crates/sp1-ntt-gadget) |
| Lattice, arXiv 2603.07947 | 2026-03-09, "Version 0.8.0" | ML-DSA-44 posted raw on-chain | None, no ZK layer | No | Tx-size model, ~14.6 tx/s, ~7.4 TB/year, native i7-12700 timings. §3.2 and §9 ECDSA figures contradict each other | **No.** Its published code URL `github.com/lattice-network/lattice` 404s | [arXiv](https://arxiv.org/abs/2603.07947) |
| starkware-bitcoin/falcon-air | created 2025-07-17; last 2025-11-27 | Falcon Z_q arithmetic, NTT/INTT | Custom Stwo AIR | Yes (arithmetic circuits) | **None** | **No LICENSE file**; self-described unaudited research prototype | [repo](https://github.com/starkware-bitcoin/falcon-air) |
| NIST PQC Signature Zoo | data 2026-04-03; repo HEAD 2026-07-22 | 116 parameter sets, native only | n/a | No | Sizes plus sign/verify cycles and us. **No keygen. x86 only. No secp256k1.** Heterogeneous `perf_source` per scheme | Yes, CC BY-SA 4.0, YAML in repo. `/wide.html` 404s | [site](https://pqshield.github.io/nist-sigs-zoo/) · [repo](https://github.com/PQShield/nist-sigs-zoo) |
| eBACS / SUPERCOP | page version 2026-07-17 | Many, native only | n/a | No | Yes, but aarch64 is Raspberry Pi class only, no Apple Silicon; `dilithium2`/`falcon512` are round-3, not FIPS-final | Yes, public methodology | [bench.cr.yp.to](https://bench.cr.yp.to/results-sign.html) |
| StarkWare PQ roadmap | 2026-06-30 | Names Falcon-512 for consensus signatures | n/a | No | No prover numbers. Explicitly non-guaranteed, governance-gated | n/a | [starkware.co](https://starkware.co/blog/the-architecture-advantage-starknets-quantum-readiness-roadmap/) |
| EF "lean Ethereum" + ethereum.org PQ page | 2025-07-31; page updated 2026-04-09 | leanXMSS for consensus | leanVM | No | No prover numbers | n/a | [EF blog](https://blog.ethereum.org/2025/07/31/lean-ethereum) · [ethereum.org](https://ethereum.org/roadmap/future-proofing/quantum-resistance/) |

## 10. Verification log

Every row was checked on **2026-07-23**. "Refuted" means our check contradicted the claim as it was handed to us.

| # | Claim checked | Verdict | Notes |
|---|---|---|---|
| 1 | BTQ/StarkWare did Falcon on Starknet **in 2024** | **Refuted** | Page says "Sep 18, 2023". Off by a year |
| 2 | BTQ completed "the first STARK-based verification of a Falcon signature" | **Refuted as phrased** | Actual wording is "post-quantum digital signatures **on Starknet**"; and the unqualified "first" is contradicted by ePrint 2021/1048 (2021) |
| 3 | BTQ demonstrated a completed STARK verification | **Not confirmed** | Page evidences a Cairo implementation plus a "Cairo profile"; never states a proof was generated |
| 4 | BTQ published performance figures | **Refuted** | No time, steps, constraints, RAM, proof size or gas anywhere on the page |
| 5 | ePrint 2021/1048 publishes prover time, RAM and proof size vs batch 128-1024 on named hardware | Confirmed | Table 3 plus "8-core Intel Core i9 processor @ 2.4 GHz with 32 GB of RAM" |
| 6 | ePrint 2021/1048 used a hash-based PQ scheme, not a FIPS scheme | Confirmed | Lamport+, w=2 WOTS over Rescue-Prime |
| 7 | ePrint 2021/1048 publishes amortized per-signature cost | **Refuted** | "amortiz", "per signature", "per-signature" appear nowhere in the 15-page PDF. Our division is an inference |
| 8 | Its code is "in facebook/winterfell" | **Refuted as stated** | Paper cites `github.com/anonauthorsub/asiaccs_2021_440`, which 404s. Winterfell link is our inference; no cross-citation exists either way |
| 9 | ePrint PDF fetchable | **Refuted** | HTTP 403 behind Cloudflare to fetchers; read via Wayback 2026-01-21 snapshot |
| 10 | Winterfell README agrees with the paper's Table 3 | **Refuted** | n=1024 at 123-bit: 20.5 s / 7.6 GB / 152 KB (README) vs 25.7 s / 9.5 GB / 165 KB (paper) |
| 11 | Amortized prover cost is flat in batch size | Confirmed, and is prior art | Stated in the Winterfell README; visible in Table 3; reproduced by ZK-ACE Table 7 and leanBench |
| 12 | s2morrow (StarkWare) has published proving benchmarks | **Refuted** | `- [ ] Stwo proving benchmarks` unchecked in the raw README on `master`; repo frozen since 2026-03-23 |
| 13 | A page summariser's reading of that roadmap | **Refuted** | Summariser returned "✅ Stwo proving benchmarks". Raw file says `- [ ]` |
| 14 | s2morrow has a batch-size sweep | **Refuted** | Makefile generates `--num_signatures 1` only |
| 15 | s2morrow's proving pipeline is reproducible | **Refuted** | Depends on `ssh://git@github.com/m-kus/proving-utils.git`, which does not resolve publicly |
| 16 | Third-party CI logs for s2morrow commit `831bb518b` | **404** | `docs.swmansion.com/maat/nightly-...` directory has rotated |
| 17 | "~9.5M L2 gas" is a StarkWare figure | **Refuted** | It is from the feltroidprime fork's marketing site; the fork's own README says ~13.2M for the same function |
| 18 | Dilithium-ZK is an ML-DSA verifier in SP1 | **Refuted** | Crate README: "not a complete ML-DSA signature verifier". It is an NTT/INTT gadget with a 4-challenge PIC |
| 19 | Dilithium-ZK is ML-DSA-44 | **Refuted** | It is ML-DSA-**65** |
| 20 | Dilithium-ZK's linked repository contains code | **Refuted** | GitHub API: `"size": 0`, no branches, `created_at == pushed_at` |
| 21 | Dilithium-ZK's published cycle figures are self-consistent | **Refuted** | Breakdown sums to 1,100,000 against a stated total of 5,625,411; the "~50%" label implies 10.3% |
| 22 | Dilithium-ZK's 22.07 s is on named hardware | **Refuted** | SP1 Network, outsourced, hardware not published. The only named hardware (Apple M1) is for native microsecond microbenchmarks |
| 23 | docs.rs "Release Date: June 13, 2026" for that crate | **Refuted as a release date** | Docs rebuild date; crate published 2025-12-08 |
| 24 | nist-sigs-zoo publishes keygen numbers | **Refuted** | No keygen column or field in the signature dataset |
| 25 | nist-sigs-zoo is a usable precision timing baseline | **Refuted** | Four different `perf_source` values across our six rows; implied clocks span 2187-3492 MHz |
| 26 | nist-sigs-zoo covers aarch64 / Apple Silicon | **Refuted** | x86 only |
| 27 | nist-sigs-zoo covers secp256k1 | **Refuted** | P-256, P-384, P-521 only |
| 28 | `pqshield.github.io/nist-sigs-zoo/wide.html` | **404** | Still indexed by search engines. Do not cite |
| 29 | ML-DSA-44 sign cycles have one authoritative value | **Refuted** | Zoo 172,926 vs pq-crystals 333,013, about 1.9x apart, both credible |
| 30 | An absolute secp256k1 verify figure on aarch64 exists | **Not found** | delvingbitcoin arm64 run gives a relative chart only, no CPU model, no units. Snippet figures were not fetched and are not asserted |
| 31 | arXiv 2603.07947 / 2603.07974 IDs are transposed | **Refuted** | Both resolve to distinct real papers submitted 2026-03-09 |
| 32 | Lattice publishes a usable ECDSA baseline | **Refuted** | §3.2, §9 and §3.4.3 give three different pairs and opposite conclusions |
| 33 | Lattice's code is available | **Refuted** | `github.com/lattice-network/lattice` returns 404 |
| 34 | ZK-ACE Table 7 was measured on an Apple M3 Pro | **Not confirmed** | Table 7 says only "Apple Silicon"; M3 Pro appears only in the Table 12 caption |
| 35 | ZK-ACE prove/tx range is 14.29-14.77 ms | **Refuted** | True range 14.29-15.76 ms, peak at N=8, non-monotone |
| 36 | ZK-ACE measures PQ signature verification in-circuit | **Refuted** | It explicitly replaces the signature object; §12.2 declines to measure in-circuit lattice verification |
| 37 | HAPPIER's chapter text | **Paywalled, unread** | Unpaywall `oa_status: "closed"`; no preprint anywhere |
| 38 | HAPPIER uses XMSS | **Refuted as stated** | Generalized XMSS from b-wagn/hash-sig, not RFC 8391 |
| 39 | HAPPIER measured aggregation of up to 2^16 signatures | **Refuted** | Every run aggregates two distinct signatures (`Bitfield: 0b00000011`); totals are a `log2(N)` extrapolation |
| 40 | HAPPIER publishes memory or hardware | **Refuted** | Zero matches for memory patterns across the artifact; no CPU named |
| 41 | HAPPIER's "2-3 MB" signature claim | **Not confirmed** | Only XMSS256-192 merge2 lands in range; the main sweep shows 7 MB to 30 MB |
| 42 | leanBench benchmarks across six proving machines | **Refuted** | One prover (leanVM), seven hardware fingerprints. The six zkVM names are an aspirational roadmap track |
| 43 | leanBench measures a FIPS scheme | **Refuted** | leanSig, generalized XMSS over Poseidon, self-labelled unaudited prototype |
| 44 | leanBench dashboard publishes readable numbers | **Refuted** | Client-side Chart.js; all figures came from committed `results/*.json` |
| 45 | leanBench is live | **Not confirmed** | Last commit 2026-06-20, newest result 2026-06-18, 1 star |
| 46 | OpenZeppelin publishes five Falcon variants with steps 97,681 to 413,819 | Confirmed | Plus five more in-`__validate__` rows to 427,598 steps, five NTT rows, and two stub rows |
| 47 | The SHAKE conformance penalty is ~3.2x | Confirmed, but understated | 3.20x for the SHAKE hint variant; **4.24x** for `shake_direct`, the closest match to a bare FIPS signature |
| 48 | OpenZeppelin's ML-DSA-44 verifier is a stub | Confirmed, and worse | It ignores `message_hash` and returns true for any 43-felt key plus 79-felt signature |
| 49 | OpenZeppelin publishes proving time | **Refuted** | Steps and L2 gas only; zero matches for `prov|time|second|ms|sec` in `results.json` |
| 50 | A page summariser's reading of that benchmark table | **Refuted** | It silently dropped both `direct` rows, including 413,819 |
| 51 | RISC Zero has no SHAKE acceleration path | **Refuted** | It patches `tiny-keccak` 2.0.2 (unstable feature); v3.0.1 notes "Stabilize bigint and keccak features" |
| 52 | Neither SP1 nor RISC Zero has a lattice or NTT precompile | Confirmed | Full SP1 syscall enum at v6.3.1 and the RISC Zero precompile table both checked |
| 53 | SP1 guest target is riscv32im | **Refuted for v6.x** | `DEFAULT_TARGET = "riscv64im-succinct-zkvm-elf"`; the SP1 "Compiling" docs page is stale and self-contradictory |
| 54 | RISC Zero and SP1 cycle counts are comparable | **Refuted** | Different ISA width, different syscall accounting, and Succinct's own docs disown cycles as a cost proxy |
| 55 | A Bonsai dollars-per-proof figure is citable | **Refuted** | Bonsai overview page 404s; Boundless docs: "As of December 2025, Bonsai is no longer available" |
| 56 | `dev.risczero.com/api/zkvm/acceleration` | **Empty body** | HTTP 200 with no content; use `/precompiles` |
| 57 | Groth16 wrapping is available on Apple Silicon | **Refuted for RISC Zero** | Docs: the Groth16 prover "only works on x86 architecture, and so Apple Silicon is currently unsupported (even via Docker)" |
| 58 | SP1 has GPU acceleration on Apple Silicon | **Refuted** | CUDA is "only supported on Linux x86_64"; AVX is Intel-only; no Metal backend documented |
| 59 | A search snippet claiming SP1 needs "at least 128GB RAM" for Groth16/PLONK | **Refuted** | Current hardware page says 16GB+ (Groth16) and 64GB+ (PLONK), wrap step "roughly 14GB ... and 60GB" |
| 60 | StarkWare published a Starknet PQ roadmap | Confirmed, and more specific than reported | 2026-06-30, three dated phases, names Falcon-512, hedged by governance |
| 61 | "Settlement is PQ, accounts are not" as a bare claim | **Not defensible unqualified** | StarkWare's own roadmap concedes Pedersen, secp syscalls and KZG inside settlement; account scheme is not protocol-fixed |
| 62 | `docs.starknet.io/learn/protocol/accounts/introduction` | **404** | Working path is `.../accounts.md` |
| 63 | An @Starknet post asserting PQ wallets are live | **Not fetchable** | X is not retrievable here. Not cited; equivalent substance taken from starkware.co and s2morrow.xyz |
| 64 | arXiv 2601.17785 timing tables | **Not extractable** | Compressed PDF streams defeated extraction; no hardware recoverable. Not cited |