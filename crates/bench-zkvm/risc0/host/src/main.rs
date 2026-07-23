//! RISC Zero host: measures the cost of verifying Ed25519 signatures in-circuit.
//!
//! The pipeline is: build a batch of real signatures on the host, execute the
//! guest to get a deterministic cycle count (no proving), then generate a real
//! proof to measure prover wall-clock, proof size, peak memory and verify time.
//! Results are written to results/zkvm/ in the shared schema.
//!
//! Every number here comes from an actual run. Cycle counts are deterministic
//! and machine independent; wall-clock and memory are tagged with the machine's
//! hardware_class and must never be plotted against a different class.

use ed25519_dalek::{Signer, SigningKey};
use pqb_core::{env as pqbenv, time};
use pqb_risc0_methods::{ED25519_VERIFY_ELF, ED25519_VERIFY_ID};
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
    // length 32 and the signature is 64 bytes. Must match the guest exactly.
    pubkey: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct GuestBatch {
    jobs: Vec<VerifyJob>,
}

/// Build N real Ed25519 signatures over the canonical transaction preimage.
fn build_batch(n: u32) -> GuestBatch {
    let mut rng = rand_core::OsRng;
    let mut jobs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let sk = SigningKey::generate(&mut rng);
        let vk = sk.verifying_key();
        let pubkey = vk.to_bytes();
        let message =
            tx_format::signing_preimage(1, 42, &pubkey, &[0xAB; 32], 1_000_000, 1_000, &[0u8; 32]);
        let signature = sk.sign(&message).to_bytes();
        jobs.push(VerifyJob {
            message,
            pubkey: pubkey.to_vec(),
            signature: signature.to_vec(),
        });
    }
    GuestBatch { jobs }
}

/// Peak resident set size of this process, in megabytes.
///
/// The prover runs in-process, so this captures its peak. getrusage is the
/// portable way to read this; ru_maxrss is bytes on macOS and kilobytes on
/// Linux, which is handled below.
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
    // CARGO_MANIFEST_DIR is crates/bench-zkvm/risc0/host.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..4 {
        p = p.parent().map(PathBuf::from).unwrap_or(p);
    }
    p
}

fn main() {
    let batch_size: u32 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(1);

    eprintln!("PQ-STARK-BENCH risc0 lane: ed25519 verify, N={batch_size}");

    let batch = build_batch(batch_size);

    // --- 1. Execute only: deterministic cycle count, no proving. ---
    eprintln!("executing guest for cycle count ...");
    let exec_env = ExecutorEnv::builder()
        .write(&batch)
        .expect("write batch")
        .build()
        .expect("build exec env");

    let session = default_executor()
        .execute(exec_env, ED25519_VERIFY_ELF)
        .expect("guest execution");

    let user_cycles = session.cycles();
    let total_cycles = session.segments.iter().map(|s| 1u64 << s.po2).sum::<u64>();
    eprintln!("  user cycles: {user_cycles}, padded cycles: {total_cycles}");

    // --- 2. Prove: warm the Metal kernels once, then measure. ---
    // The first proof on Apple Silicon JIT-compiles GPU kernels, so its wall
    // time is not representative. It is discarded per the methodology.
    eprintln!("proving (warmup, discarded) ...");
    let warm_env = ExecutorEnv::builder()
        .write(&batch)
        .expect("write batch")
        .build()
        .expect("build env");
    let _ = default_prover()
        .prove_with_opts(warm_env, ED25519_VERIFY_ELF, &ProverOpts::default())
        .expect("warmup proof");

    eprintln!("proving (measured) ...");
    let prove_env = ExecutorEnv::builder()
        .write(&batch)
        .expect("write batch")
        .build()
        .expect("build env");
    let t0 = Instant::now();
    let prove_info = default_prover()
        .prove_with_opts(prove_env, ED25519_VERIFY_ELF, &ProverOpts::default())
        .expect("measured proof");
    let prove_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let peak_ram_mb = peak_rss_mb();

    let receipt = prove_info.receipt;
    let proof_bytes = bincode::serialize(&receipt).map(|v| v.len()).ok();

    // --- 3. Verify, and check the guest actually verified every signature. ---
    let t1 = Instant::now();
    receipt.verify(ED25519_VERIFY_ID).expect("receipt verifies");
    let verify_ms = t1.elapsed().as_secs_f64() * 1000.0;

    let (n_committed, verified): (u32, u32) = receipt.journal.decode().expect("decode journal");
    assert_eq!(n_committed, batch_size, "guest saw a different batch size");
    assert_eq!(
        verified, batch_size,
        "guest did not verify every signature: {verified}/{batch_size}"
    );
    eprintln!("  proved that {verified}/{batch_size} ed25519 signatures verify");

    // --- 4. Assemble and write the results file. ---
    let environment = pqbenv::capture();
    let backend = if environment.hardware_class == "local-m3max" {
        "metal"
    } else {
        "cpu-scalar"
    };

    let mut toolchain = BTreeMap::new();
    // cargo-risczero is a cargo subcommand, not a standalone binary on PATH.
    toolchain.insert(
        "cargo-risczero".to_string(),
        tool_version("cargo", &["risczero", "--version"]),
    );
    toolchain.insert("r0vm".to_string(), tool_version("r0vm", &["--version"]));
    toolchain.insert("rustc".to_string(), environment.rustc.clone());

    let workload = Workload {
        scheme: Scheme {
            name: "ed25519".into(),
            family: Family::Classical,
            spec: "RFC 8032 (classical, not post-quantum)".into(),
            crate_name: "ed25519-dalek".into(),
            crate_version: "2.1".into(),
            hash_primitive: "SHA-512".into(),
            conformant: true,
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
            proof_mode: "composite".into(),
            security_bits: SecurityBits {
                value: Some(96),
                kind: SecurityKind::Claimed,
                source:
                    "https://dev.risczero.com/api/security-model, accessed 2026-07-23"
                        .into(),
            },
            segment_limit_po2: session.segments.first().map(|s| s.po2 as u32),
            precompiles_used: vec![],
            precompile_assert_passed: false,
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
            prove_ms: Some(prove_ms),
            peak_ram_mb: Some(peak_ram_mb),
            proof_bytes,
            verify_ms: Some(verify_ms),
        },
        status: Status::Ok,
        abort_reason: None,
        caveats: vec![
            "stock ed25519-dalek: no curve25519 precompile, this is the honest worst case".into(),
            "composite receipt: proof size grows with segment count, not constant".into(),
            "wall-clock and peak RAM are hardware-class local-m3max, not comparable across machines".into(),
        ],
    };

    let now = SystemTime::now();
    let host = pqbenv::host_label();
    let run_id = format!("{}-risc0", time::run_id(now, &host));

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
        "  ed25519 N={batch_size}: {user_cycles} cycles, prove {prove_ms:.0} ms, verify {verify_ms:.1} ms, proof {} B, peak {peak_ram_mb:.0} MB",
        proof_bytes.unwrap_or(0)
    );
    eprintln!("wrote {}", out_path.display());
}
