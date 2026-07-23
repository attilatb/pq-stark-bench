//! RISC Zero host: measures the cost of verifying signatures in-circuit.
//!
//! Usage: risc0-bench <scheme> <mode> [N]
//!   scheme: ed25519 | falcon512 | mldsa44
//!   mode:   execute | prove
//!   N:      batch size, default 1
//!
//! `execute` runs the guest for a deterministic cycle count and nothing else,
//! which is fast and machine independent. `prove` additionally generates a real
//! proof and measures prover wall-clock, proof size, peak memory and verify
//! time, which are hardware dependent and tagged with the machine's class.
//!
//! Every number here comes from an actual run.

use pqb_core::{env as pqbenv, time};
use pqb_risc0_methods as methods;
use pqb_zkvm_common::{
    Batch, Cost, Family, Prover, Scheme, SecurityBits, SecurityKind, Status, Topology, Workload,
    ZkvmResultsFile,
};
use risc0_zkvm::{default_executor, default_prover, ExecutorEnv, ProverOpts};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime};

#[derive(Serialize, Deserialize)]
struct VerifyJob {
    message: Vec<u8>,
    // Byte vectors, not fixed arrays: serde's derive only covers arrays up to
    // length 32. Must match the guest exactly.
    pubkey: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct GuestBatch {
    jobs: Vec<VerifyJob>,
}

/// The canonical transaction preimage that every scheme signs.
fn canonical_message(pubkey: &[u8]) -> Vec<u8> {
    tx_format::signing_preimage(1, 42, pubkey, &[0xAB; 32], 1_000_000, 1_000, &[0u8; 32])
}

/// Static description of a scheme and the guest that verifies it.
struct SchemeDef {
    name: &'static str,
    family: Family,
    spec: &'static str,
    crate_name: &'static str,
    crate_version: &'static str,
    hash_primitive: &'static str,
    conformant: bool,
    elf: &'static [u8],
    image_id: [u32; 8],
    /// Precompiles this build is expected to use. Empty for stock builds.
    precompiles: &'static [&'static str],
    /// For accelerated builds, the cycle count must come in below this to prove
    /// the precompile actually engaged (assert, do not assume). None for stock.
    accel_assert_below: Option<u64>,
    /// Extra disclosures beyond the shared ones.
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
            conformant: true,
            elf: methods::ED25519_VERIFY_ELF,
            image_id: methods::ED25519_VERIFY_ID,
            precompiles: &[],
            accel_assert_below: None,
            caveats: &["stock ed25519-dalek: no curve25519 precompile, the honest worst case"],
        },
        "ed25519accel" => SchemeDef {
            name: "ed25519-accel",
            family: Family::Classical,
            spec: "RFC 8032 (classical, not post-quantum)",
            crate_name: "ed25519-dalek + risc0 curve25519-dalek fork",
            crate_version: "2.1 / curve25519-4.1.3-risczero.0",
            hash_primitive: "SHA-512",
            conformant: true,
            elf: methods::ED25519_ACCEL_VERIFY_ELF,
            image_id: methods::ED25519_ACCEL_VERIFY_ID,
            precompiles: &["curve25519"],
            // Stock ed25519 verify is ~3.24M cycles; the precompile must bring
            // it well below 2M or it did not engage.
            accel_assert_below: Some(2_000_000),
            caveats: &[
                "accelerated: curve25519 field arithmetic routed to the RISC Zero precompile",
                "this is the classical control WITH its precompile, unlike the stock ed25519 row",
            ],
        },
        "falcon512" => SchemeDef {
            name: "falcon-512",
            family: Family::Lattice,
            spec: "NIST selected. FIPS 206 not published as of 2026-07-23.",
            crate_name: "fn-dsa-vrfy",
            crate_version: "0.4.0",
            hash_primitive: "SHAKE-256",
            conformant: true,
            elf: methods::FALCON512_VERIFY_ELF,
            image_id: methods::FALCON512_VERIFY_ID,
            precompiles: &[],
            accel_assert_below: None,
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
            conformant: true,
            elf: methods::MLDSA44_VERIFY_ELF,
            image_id: methods::MLDSA44_VERIFY_ID,
            precompiles: &[],
            accel_assert_below: None,
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
            conformant: true,
            elf: methods::SLHDSA128S_VERIFY_ELF,
            image_id: methods::SLHDSA128S_VERIFY_ID,
            precompiles: &[],
            accel_assert_below: None,
            caveats: &[
                "stock: no SHA-256 precompile is routed to fips205, so it runs as plain RISC-V",
                "hash-based: verification is dominated by SHA-256",
            ],
        },
        other => panic!(
            "unknown scheme: {other}. Use ed25519 | ed25519accel | falcon512 | mldsa44 | slhdsa128s"
        ),
    }
}

/// Build N real signatures for the given scheme, signing on the host.
fn build_batch(scheme: &str, n: u32) -> GuestBatch {
    match scheme {
        "ed25519" | "ed25519accel" => build_ed25519(n),
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
        // Hedged (randomized) signing is the FIPS 205 default.
        let sig = sk.try_sign(&message, b"", true).expect("slh-dsa sign");
        jobs.push(VerifyJob {
            message,
            pubkey,
            signature: sig.to_vec(),
        });
    }
    GuestBatch { jobs }
}

/// Peak resident set size of this process, in megabytes.
fn peak_rss_mb() -> f64 {
    // SAFETY: getrusage with a stack-allocated, zeroed rusage is a read-only
    // syscall with no aliasing or lifetime concerns.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
    if rc != 0 {
        return f64::NAN;
    }
    let maxrss = usage.ru_maxrss as f64;
    if cfg!(target_os = "macos") {
        maxrss / 1024.0 / 1024.0
    } else {
        maxrss / 1024.0
    }
}

fn tool_version(bin: &str, args: &[&str]) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("{home}/.risc0/bin/{bin}"),
        format!("{home}/.cargo/bin/{bin}"),
        bin.to_string(),
    ];
    for c in candidates {
        if let Ok(out) = Command::new(&c).args(args).output() {
            if out.status.success() {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    let line = s.lines().next().unwrap_or("").trim().to_string();
                    if !line.is_empty() {
                        return line;
                    }
                }
            }
        }
    }
    "unknown".to_string()
}

fn repo_root() -> PathBuf {
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
        "PQ-STARK-BENCH risc0 lane: {} verify, mode={mode}, N={batch_size}",
        def.name
    );

    let batch = build_batch(&scheme_arg, batch_size);

    // --- Execute: deterministic cycle count, no proving. ---
    eprintln!("executing guest for cycle count ...");
    let exec_env = ExecutorEnv::builder()
        .write(&batch)
        .expect("write batch")
        .build()
        .expect("build exec env");
    let session = default_executor()
        .execute(exec_env, def.elf)
        .expect("guest execution");
    let user_cycles = session.cycles();
    let total_cycles = session.segments.iter().map(|s| 1u64 << s.po2).sum::<u64>();
    let (n_exec, verified_exec): (u32, u32) = session.journal.decode().expect("decode journal");
    assert_eq!(n_exec, batch_size, "guest saw a different batch size");
    assert_eq!(
        verified_exec, batch_size,
        "guest did not verify every signature: {verified_exec}/{batch_size}"
    );
    eprintln!("  user cycles: {user_cycles}, padded cycles: {total_cycles}");

    // Assert the precompile actually engaged, do not assume it. An accelerated
    // build whose cycle count is not well below the stock threshold means the
    // patch silently did nothing, which would inflate every downstream chart.
    let precompile_assert_passed = match def.accel_assert_below {
        Some(per_sig_threshold) => {
            // The threshold is per signature, so it scales with the batch size.
            let per_sig = user_cycles / batch_size.max(1) as u64;
            assert!(
                per_sig < per_sig_threshold,
                "{} declared a precompile but ran {per_sig} cycles per signature, not below \
                 {per_sig_threshold}: the precompile did not engage",
                def.name
            );
            true
        }
        None => false,
    };

    // --- Prove (optional): wall-clock, proof size, peak RAM, verify. ---
    let mut prove_ms = None;
    let mut peak_ram_mb = None;
    let mut proof_bytes = None;
    let mut verify_ms = None;

    if proving {
        // Warm the Metal kernels once; the first proof JIT-compiles them.
        eprintln!("proving (warmup, discarded) ...");
        let warm_env = ExecutorEnv::builder()
            .write(&batch)
            .expect("write")
            .build()
            .expect("env");
        let _ = default_prover()
            .prove_with_opts(warm_env, def.elf, &ProverOpts::default())
            .expect("warmup proof");

        eprintln!("proving (measured) ...");
        let prove_env = ExecutorEnv::builder()
            .write(&batch)
            .expect("write")
            .build()
            .expect("env");
        let t0 = Instant::now();
        let prove_info = default_prover()
            .prove_with_opts(prove_env, def.elf, &ProverOpts::default())
            .expect("measured proof");
        prove_ms = Some(t0.elapsed().as_secs_f64() * 1000.0);
        peak_ram_mb = Some(peak_rss_mb());

        let receipt = prove_info.receipt;
        proof_bytes = bincode::serialize(&receipt).map(|v| v.len()).ok();

        let t1 = Instant::now();
        receipt.verify(def.image_id).expect("receipt verifies");
        verify_ms = Some(t1.elapsed().as_secs_f64() * 1000.0);

        let (_n, verified): (u32, u32) = receipt.journal.decode().expect("journal");
        assert_eq!(verified, batch_size, "proof did not verify every signature");
        eprintln!(
            "  proved that {verified}/{batch_size} {} signatures verify",
            def.name
        );
    }

    // --- Assemble and write the results file. ---
    let environment = pqbenv::capture();
    let backend = if !proving {
        "none"
    } else if environment.hardware_class == "local-m3max" {
        "metal"
    } else {
        "cpu-scalar"
    };

    let mut toolchain = BTreeMap::new();
    toolchain.insert(
        "cargo-risczero".to_string(),
        tool_version("cargo", &["risczero", "--version"]),
    );
    toolchain.insert("r0vm".to_string(), tool_version("r0vm", &["--version"]));
    toolchain.insert("rustc".to_string(), environment.rustc.clone());

    let mut caveats = vec!["cycle counts are deterministic and machine independent".to_string()];
    caveats.extend(def.caveats.iter().map(|c| c.to_string()));
    if proving {
        caveats.push(
            "wall-clock and peak RAM are hardware-class local-m3max, not comparable across machines"
                .to_string(),
        );
        caveats.push(
            "composite receipt: proof size grows with segment count, not constant".to_string(),
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
            conformant: def.conformant,
            prerelease: false,
        },
        prover: Prover {
            name: "risc0".into(),
            version: tool_version("r0vm", &["--version"])
                .split_whitespace()
                .last()
                .unwrap_or("unknown")
                .to_string(),
            isa: "riscv32im".into(),
            backend: backend.into(),
            proof_mode: if proving { "composite" } else { "execute" }.into(),
            security_bits: SecurityBits {
                value: Some(96),
                kind: SecurityKind::Claimed,
                source: "https://dev.risczero.com/api/security-model, accessed 2026-07-23".into(),
            },
            segment_limit_po2: session.segments.first().map(|s| s.po2),
            precompiles_used: def.precompiles.iter().map(|p| p.to_string()).collect(),
            precompile_assert_passed,
        },
        batch: Batch {
            n: batch_size,
            topology: Topology::Flat,
            arity: None,
        },
        cost: Cost {
            cycles: Some(user_cycles),
            cycles_source: Some("SessionInfo::cycles".into()),
            total_cycles: Some(total_cycles),
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
        "{}-risc0-{}-{}-n{}",
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
        "  {} N={batch_size}: {user_cycles} cycles{}",
        def.name,
        if proving {
            format!(
                ", prove {:.0} ms, verify {:.1} ms, proof {} B, peak {:.0} MB",
                prove_ms.unwrap_or(0.0),
                verify_ms.unwrap_or(0.0),
                proof_bytes.unwrap_or(0),
                peak_ram_mb.unwrap_or(0.0)
            )
        } else {
            String::new()
        }
    );
    eprintln!("wrote {}", out_path.display());
}
