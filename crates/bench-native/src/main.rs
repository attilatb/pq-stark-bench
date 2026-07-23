//! PQ-STARK-BENCH native benchmark runner.
//!
//! Measures keygen, sign and verify for each scheme and writes a machine
//! readable results file conforming to the schema in KICKOFF.md section 6.
//!
//! Every number written here comes from an actual run on this machine. Nothing
//! is estimated, extrapolated, or copied from a reference table.

#![forbid(unsafe_code)]

mod schemes;

use pqb_core::{env, stats, time};
use schemes::{Budget, SchemeMeasurement};
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

/// One row of the results array, matching the published schema.
#[derive(Debug, Serialize)]
struct ResultRow {
    scheme: String,
    family: schemes::Family,
    standard: String,
    implementation: String,
    operation: &'static str,
    batch_size: usize,

    median_ns: u64,
    p95_ns: u64,
    iterations: usize,

    // Additive detail beyond the base schema. The site ignores unknown fields.
    min_ns: u64,
    max_ns: u64,
    mean_ns: u64,
    warmup_iterations: usize,

    sig_bytes: usize,
    pubkey_bytes: usize,

    // Null for native runs. Populated by the zkVM harness in Phase 2.
    proof_bytes: Option<usize>,
    prover_cycles: Option<u64>,
    peak_ram_mb: Option<f64>,
}

#[derive(Debug, Serialize)]
struct TxSizeRow {
    scheme: String,
    model: &'static str,
    on_chain_bytes: usize,
    witness_bytes: usize,
    total_bytes: usize,
}

#[derive(Debug, Serialize)]
struct ResultsFile {
    run_id: String,
    kind: &'static str,
    schema_version: u32,
    generated_at: String,
    environment: env::Environment,
    /// Per-transaction byte accounting under each publication model.
    tx_sizes: Vec<TxSizeRow>,
    results: Vec<ResultRow>,
}

fn rows_for(m: &SchemeMeasurement) -> Vec<ResultRow> {
    let mk = |operation: &'static str, t: &stats::Timing| ResultRow {
        scheme: m.scheme.clone(),
        family: m.family,
        standard: m.standard.clone(),
        implementation: m.implementation.clone(),
        operation,
        // Native measurements are per single signature.
        batch_size: 1,
        median_ns: t.median_ns,
        p95_ns: t.p95_ns,
        iterations: t.iterations,
        min_ns: t.min_ns,
        max_ns: t.max_ns,
        mean_ns: t.mean_ns,
        warmup_iterations: t.warmup_iterations,
        sig_bytes: m.sig_bytes,
        pubkey_bytes: m.pubkey_bytes,
        proof_bytes: None,
        prover_cycles: None,
        peak_ram_mb: None,
    };

    vec![
        mk("keygen", &m.keygen),
        mk("sign", &m.sign),
        mk("verify", &m.verify),
    ]
}

fn tx_rows_for(m: &SchemeMeasurement) -> Vec<TxSizeRow> {
    tx_format::tx_sizes(m.pubkey_bytes, m.sig_bytes)
        .into_iter()
        .map(|s| TxSizeRow {
            scheme: m.scheme.clone(),
            model: s.model.as_str(),
            on_chain_bytes: s.on_chain_bytes,
            witness_bytes: s.witness_bytes,
            total_bytes: s.total_bytes,
        })
        .collect()
}

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/bench-native.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn main() -> std::io::Result<()> {
    let quick = std::env::args().any(|a| a == "--quick");
    let budget = if quick {
        Budget::quick()
    } else {
        Budget::default()
    };

    let environment = env::capture();

    eprintln!("PQ-STARK-BENCH native benchmark");
    eprintln!("  cpu            {}", environment.cpu);
    eprintln!("  cores          {}", environment.cores);
    eprintln!("  ram_gb         {}", environment.ram_gb);
    eprintln!("  os             {}", environment.os);
    eprintln!("  rustc          {}", environment.rustc);
    eprintln!("  hardware_class {}", environment.hardware_class);
    eprintln!(
        "  budget         warmup={} min_iters={} max_iters={} per_op_budget={:?}",
        budget.warmup, budget.min_iterations, budget.max_iterations, budget.per_op_budget
    );
    if quick {
        eprintln!("  MODE           quick. Below the N >= 100 floor, not for publication.");
    }
    eprintln!();

    type BenchFn = fn(Budget) -> SchemeMeasurement;
    let suite: &[(&str, BenchFn)] = &[
        ("ed25519", schemes::bench_ed25519),
        ("ecdsa-secp256k1", schemes::bench_ecdsa_secp256k1),
        ("ml-dsa-44", schemes::bench_ml_dsa_44),
        ("falcon-512", schemes::bench_falcon_512),
        ("slh-dsa-sha2-128s", schemes::bench_slh_dsa_sha2_128s),
        ("slh-dsa-sha2-128f", schemes::bench_slh_dsa_sha2_128f),
    ];

    let mut measurements: Vec<SchemeMeasurement> = Vec::new();
    for (name, f) in suite {
        eprintln!("measuring {name} ...");
        let started = SystemTime::now();
        let m = f(budget);
        let secs = started
            .elapsed()
            .map(|d| d.as_secs_f64())
            .unwrap_or(f64::NAN);
        eprintln!("  done in {secs:.1}s");
        measurements.push(m);
    }

    let results: Vec<ResultRow> = measurements.iter().flat_map(rows_for).collect();
    let tx_sizes: Vec<TxSizeRow> = measurements.iter().flat_map(tx_rows_for).collect();

    let now = SystemTime::now();
    let ts = time::rfc3339_utc(now);
    let host = env::host_label();
    let run_id = time::run_id(now, &host);

    let file = ResultsFile {
        run_id: run_id.clone(),
        kind: "native",
        schema_version: 1,
        generated_at: ts.clone(),
        environment,
        tx_sizes,
        results,
    };

    let out_dir = repo_root().join("results").join("native");
    std::fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{run_id}.json"));

    let json = serde_json::to_string_pretty(&file).expect("results must serialize");
    let mut f = std::fs::File::create(&out_path)?;
    f.write_all(json.as_bytes())?;
    f.write_all(b"\n")?;

    eprintln!();
    for m in &measurements {
        eprintln!(
            "  {:<18} keygen {:>10} ns   sign {:>10} ns   verify {:>10} ns   (pk {} B, sig {} B)",
            m.scheme,
            m.keygen.median_ns,
            m.sign.median_ns,
            m.verify.median_ns,
            m.pubkey_bytes,
            m.sig_bytes
        );
    }
    eprintln!();
    eprintln!("wrote {}", out_path.display());

    Ok(())
}
