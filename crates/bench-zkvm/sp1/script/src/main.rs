//! SP1 host: measures the cost of verifying signatures in-circuit.
//!
//! Usage: sp1-bench <scheme> <mode> [N]
//!   scheme: ed25519 | falcon512 | mldsa44
//!   mode:   execute | prove
//!   N:      batch size, default 1
//!
//! `execute` runs the guest for a deterministic instruction (cycle) count and
//! nothing else. `prove` additionally generates a proof and measures wall-clock,
//! proof size, peak memory and verify time. SP1 has no GPU path on Apple
//! Silicon, so proving here is scalar CPU and slow; cycle counts are the cheap
//! and machine-independent signal.

use pqb_core::{env as pqbenv, time};
use pqb_zkvm_common::{
    Batch, Cost, Family, Prover, Scheme, SecurityBits, SecurityKind, Status, Topology, Workload,
    ZkvmResultsFile,
};
use serde::{Deserialize, Serialize};
// Aliased because pqb_zkvm_common also exports a `Prover` (the results-schema
// struct). This one is the SP1 trait that provides execute/prove/setup/verify.
use sp1_sdk::blocking::{Prover as _Sp1Prover, ProverClient};
use sp1_sdk::{include_elf, Elf, SP1Stdin};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

/// The ELF for a scheme's guest. Elf is cheap to clone, so this is called at
/// each use site (execute, and again for proving).
fn elf_for(scheme: &str) -> Elf {
    match scheme {
        "ed25519" => include_elf!("ed25519_verify"),
        "falcon512" => include_elf!("falcon512_verify"),
        "mldsa44" => include_elf!("mldsa44_verify"),
        "slhdsa128s" => include_elf!("slhdsa128s_verify"),
        other => panic!("unknown scheme: {other}"),
    }
}

#[derive(Serialize, Deserialize)]
struct VerifyJob {
    message: Vec<u8>,
    pubkey: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct GuestBatch {
    jobs: Vec<VerifyJob>,
}

fn canonical_message(pubkey: &[u8]) -> Vec<u8> {
    tx_format::signing_preimage(1, 42, pubkey, &[0xAB; 32], 1_000_000, 1_000, &[0u8; 32])
}

struct SchemeDef {
    name: &'static str,
    family: Family,
    spec: &'static str,
    crate_name: &'static str,
    crate_version: &'static str,
    hash_primitive: &'static str,
    caveats: &'static [&'static str],
}

fn scheme_def(scheme: &str) -> SchemeDef {
    match scheme {
        "ed25519" => SchemeDef {
            name: "ed25519",
            family: Family::Classical,
            spec: "RFC 8032 (classical, not post-quantum)",
            crate_name: "ed25519-dalek",
            crate_version: "2.1",
            hash_primitive: "SHA-512",
            caveats: &["stock ed25519-dalek: no curve25519 precompile, the honest worst case"],
        },
        "falcon512" => SchemeDef {
            name: "falcon-512",
            family: Family::Lattice,
            spec: "NIST selected. FIPS 206 not published as of 2026-07-23.",
            crate_name: "fn-dsa-vrfy",
            crate_version: "0.4.0",
            hash_primitive: "SHAKE-256",
            caveats: &[
                "verification is integer-only; signing is done on the host",
                "no lattice or NTT precompile on this prover",
            ],
        },
        "mldsa44" => SchemeDef {
            name: "ml-dsa-44",
            family: Family::Lattice,
            spec: "NIST FIPS 204",
            crate_name: "fips204",
            crate_version: "0.4.6",
            hash_primitive: "SHAKE-256",
            caveats: &[
                "stock: no SHAKE precompile reaches fips204, so SHAKE runs as plain RISC-V",
                "no lattice or NTT precompile on this prover",
            ],
        },
        "slhdsa128s" => SchemeDef {
            name: "slh-dsa-sha2-128s",
            family: Family::Hash,
            spec: "NIST FIPS 205",
            crate_name: "fips205",
            crate_version: "0.4.1",
            hash_primitive: "SHA-256",
            caveats: &[
                "stock: no SHA-256 precompile is routed to fips205, so it runs as plain RISC-V",
                "hash-based: verification is dominated by SHA-256",
            ],
        },
        other => panic!("unknown scheme: {other}. Use ed25519 | falcon512 | mldsa44 | slhdsa128s"),
    }
}

fn build_batch(scheme: &str, n: u32) -> GuestBatch {
    match scheme {
        "ed25519" => build_ed25519(n),
        "falcon512" => build_falcon512(n),
        "mldsa44" => build_mldsa44(n),
        "slhdsa128s" => build_slhdsa128s(n),
        other => panic!("unknown scheme: {other}"),
    }
}

fn build_ed25519(n: u32) -> GuestBatch {
    use ed25519_dalek::{Signer, SigningKey};
    let mut rng = rand_core::OsRng;
    let mut jobs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let sk = SigningKey::generate(&mut rng);
        let pubkey = sk.verifying_key().to_bytes();
        let message = canonical_message(&pubkey);
        let signature = sk.sign(&message).to_bytes();
        jobs.push(VerifyJob {
            message,
            pubkey: pubkey.to_vec(),
            signature: signature.to_vec(),
        });
    }
    GuestBatch { jobs }
}

fn build_falcon512(n: u32) -> GuestBatch {
    use fn_dsa::{
        sign_key_size, signature_size, vrfy_key_size, KeyPairGenerator, KeyPairGeneratorStandard,
        SigningKey, SigningKeyStandard, DOMAIN_NONE, FN_DSA_LOGN_512, HASH_ID_RAW,
    };
    let logn = FN_DSA_LOGN_512;
    let mut jobs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let mut kg = KeyPairGeneratorStandard::default();
        let mut sk_buf = vec![0u8; sign_key_size(logn)];
        let mut vk_buf = vec![0u8; vrfy_key_size(logn)];
        kg.keygen(logn, &mut rand_core::OsRng, &mut sk_buf, &mut vk_buf);
        let message = canonical_message(&vk_buf);
        let mut sk = SigningKeyStandard::decode(&sk_buf).expect("falcon signing key");
        let mut sig = vec![0u8; signature_size(logn)];
        sk.sign(
            &mut rand_core::OsRng,
            &DOMAIN_NONE,
            &HASH_ID_RAW,
            &message,
            &mut sig,
        );
        jobs.push(VerifyJob {
            message,
            pubkey: vk_buf,
            signature: sig,
        });
    }
    GuestBatch { jobs }
}

fn build_mldsa44(n: u32) -> GuestBatch {
    use fips204::ml_dsa_44;
    use fips204::traits::{SerDes, Signer};
    let mut jobs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let (pk, sk) = ml_dsa_44::try_keygen().expect("ml-dsa-44 keygen");
        let pubkey = pk.into_bytes().to_vec();
        let message = canonical_message(&pubkey);
        let sig = sk.try_sign(&message, b"").expect("ml-dsa-44 sign");
        jobs.push(VerifyJob {
            message,
            pubkey,
            signature: sig.to_vec(),
        });
    }
    GuestBatch { jobs }
}

fn build_slhdsa128s(n: u32) -> GuestBatch {
    use fips205::slh_dsa_sha2_128s;
    use fips205::traits::{SerDes, Signer};
    let mut jobs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let (pk, sk) = slh_dsa_sha2_128s::try_keygen().expect("slh-dsa keygen");
        let pubkey = pk.into_bytes().to_vec();
        let message = canonical_message(&pubkey);
        let sig = sk.try_sign(&message, b"", true).expect("slh-dsa sign");
        jobs.push(VerifyJob {
            message,
            pubkey,
            signature: sig.to_vec(),
        });
    }
    GuestBatch { jobs }
}

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/bench-zkvm/sp1/script.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..4 {
        p = p.parent().map(PathBuf::from).unwrap_or(p);
    }
    p
}

fn main() {
    let scheme_arg = std::env::args().nth(1).unwrap_or_else(|| "ed25519".into());
    let mode = std::env::args().nth(2).unwrap_or_else(|| "execute".into());
    let batch_size: u32 = std::env::args()
        .nth(3)
        .and_then(|a| a.parse().ok())
        .unwrap_or(1);

    let def = scheme_def(&scheme_arg);
    let proving = mode == "prove";

    eprintln!(
        "PQ-STARK-BENCH sp1 lane: {} verify, mode={mode}, N={batch_size}",
        def.name
    );

    // Proving on SP1 has no GPU path on Apple Silicon, so a cross-prover
    // wall-clock comparison must run on Linux x86 where both provers have
    // comparable backends (see docs/METHODOLOGY.md). This lane therefore
    // measures cycle counts only, which are deterministic and machine
    // independent. `prove` is intentionally not implemented here.
    if proving {
        panic!(
            "SP1 proving is deferred to the Linux x86 fairness run; this lane is execute-only. \
             Use mode=execute."
        );
    }

    let batch = build_batch(&scheme_arg, batch_size);
    let mut stdin = SP1Stdin::new();
    stdin.write(&batch);

    let client = ProverClient::from_env();

    // --- Execute: deterministic instruction count, no proving. ---
    eprintln!("executing guest for cycle count ...");
    let (public_values, report) = client
        .execute(elf_for(&scheme_arg), stdin)
        .run()
        .expect("guest execution");
    let cycles = report.total_instruction_count();
    eprintln!("  instruction count (cycles): {cycles}");

    // Read the committed values back in the order the guest committed them.
    let mut pv = public_values;
    let n_committed = pv.read::<u32>();
    let verified = pv.read::<u32>();
    assert_eq!(n_committed, batch_size, "guest saw a different batch size");
    assert_eq!(
        verified, batch_size,
        "guest did not verify every signature: {verified}/{batch_size}"
    );

    let prove_ms: Option<f64> = None;
    let peak_ram_mb: Option<f64> = None;
    let proof_bytes: Option<usize> = None;
    let verify_ms: Option<f64> = None;

    // --- Assemble and write the results file. ---
    let environment = pqbenv::capture();
    let backend = if !proving { "none" } else { "cpu-scalar" };

    let mut toolchain = BTreeMap::new();
    toolchain.insert("sp1".to_string(), "6.3.1".to_string());
    toolchain.insert("rustc".to_string(), environment.rustc.clone());

    let mut caveats = vec![
        "cycle counts are deterministic and machine independent".to_string(),
        "SP1 cycles are RV64IM instruction counts, not comparable to RISC Zero cycles".to_string(),
    ];
    caveats.extend(def.caveats.iter().map(|c| c.to_string()));
    if proving {
        caveats.push(
            "SP1 has no GPU path on Apple Silicon, so this wall-clock is scalar CPU".to_string(),
        );
        caveats.push(
            "wall-clock and peak RAM are hardware-class local-m3max, not comparable across machines"
                .to_string(),
        );
    }

    let workload = Workload {
        scheme: Scheme {
            name: def.name.into(),
            family: def.family,
            spec: def.spec.into(),
            crate_name: def.crate_name.into(),
            crate_version: def.crate_version.into(),
            hash_primitive: def.hash_primitive.into(),
            conformant: true,
            prerelease: false,
        },
        prover: Prover {
            name: "sp1".into(),
            version: "6.3.1".into(),
            isa: "riscv64im".into(),
            backend: backend.into(),
            proof_mode: if proving { "core" } else { "execute" }.into(),
            security_bits: SecurityBits {
                value: None,
                kind: SecurityKind::Unknown,
                source: "SP1 does not publish a single conjectured soundness figure for all modes"
                    .into(),
            },
            segment_limit_po2: None,
            precompiles_used: vec![],
            precompile_assert_passed: false,
        },
        batch: Batch {
            n: batch_size,
            topology: Topology::Flat,
            arity: None,
        },
        cost: Cost {
            cycles: Some(cycles),
            cycles_source: Some("ExecutionReport::total_instruction_count".into()),
            total_cycles: None,
            prove_ms,
            peak_ram_mb,
            proof_bytes,
            verify_ms,
        },
        status: Status::Ok,
        abort_reason: None,
        caveats,
    };

    let now = SystemTime::now();
    let host = pqbenv::host_label();
    let run_id = format!(
        "{}-sp1-{}-{}-n{}",
        time::run_id(now, &host),
        def.name,
        mode,
        batch_size
    );

    let file = ZkvmResultsFile {
        run_id: run_id.clone(),
        kind: "zkvm".into(),
        schema_version: 1,
        generated_at: time::rfc3339_utc(now),
        environment,
        toolchain,
        workloads: vec![workload],
    };

    let out_dir = repo_root().join("results").join("zkvm");
    std::fs::create_dir_all(&out_dir).expect("create results dir");
    let out_path = out_dir.join(format!("{run_id}.json"));
    let mut f = std::fs::File::create(&out_path).expect("create results file");
    f.write_all(file.to_json().as_bytes())
        .expect("write results");

    eprintln!();
    eprintln!(
        "  {} N={batch_size}: {cycles} cycles (sp1 RV64IM)",
        def.name
    );
    eprintln!("wrote {}", out_path.display());
}
