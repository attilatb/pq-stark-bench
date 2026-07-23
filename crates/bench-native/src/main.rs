//! PQ-STARK-BENCH native benchmark runner.
//!
//! Measures keygen, sign and verify for each scheme and writes a machine
//! readable results file conforming to the schema in KICKOFF.md section 6.
//!
//! Every number written here comes from an actual run on this machine. Nothing
//! is estimated, extrapolated, or copied from a reference table.

#![forbid(unsafe_code)]

mod env;
mod schemes;
mod stats;

use schemes::{Budget, SchemeMeasurement};
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn rfc3339_utc(now: SystemTime) -> String {
    // Minimal, dependency-free UTC formatting. Civil-date conversion uses the
    // standard days-from-civil algorithm.
    let secs = now
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs();

    let days = (secs / 86_400) as i64;
    let tod = secs % 86_400;
    let (h, mi, s) = (tod / 3600, (tod % 3600) / 60, tod % 60);
    let (y, mo, d) = civil_from_days(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Howard Hinnant's days-from-civil inverse, ported directly.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
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
        Budget {
            warmup: 5,
            iterations: 25,
            keygen_iterations: 25,
        }
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
        "  budget         warmup={} iterations={} keygen={}",
        budget.warmup, budget.iterations, budget.keygen_iterations
    );
    eprintln!();

    let mut measurements: Vec<SchemeMeasurement> = Vec::new();

    eprintln!("measuring ed25519 ...");
    measurements.push(schemes::bench_ed25519(budget));

    eprintln!("measuring ecdsa-secp256k1 ...");
    measurements.push(schemes::bench_ecdsa_secp256k1(budget));

    let results: Vec<ResultRow> = measurements.iter().flat_map(rows_for).collect();
    let tx_sizes: Vec<TxSizeRow> = measurements.iter().flat_map(tx_rows_for).collect();

    let now = SystemTime::now();
    let ts = rfc3339_utc(now);
    let host = env::host_label();
    let run_id = format!("{}-{}", ts.replace(':', "-"), host);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_matches_known_dates() {
        // 1970-01-01 is day 0.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-03-01 is day 11017.
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
        // 2026-07-23 is day 20657 (cross-checked against an independent
        // implementation, not assumed).
        assert_eq!(civil_from_days(20_657), (2026, 7, 23));
        assert_eq!(civil_from_days(20_658), (2026, 7, 24));
        // Leap-day boundary, the usual source of off-by-one errors here.
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    }

    #[test]
    fn rfc3339_formats_epoch() {
        assert_eq!(rfc3339_utc(UNIX_EPOCH), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn run_id_is_filesystem_safe() {
        let ts = rfc3339_utc(UNIX_EPOCH).replace(':', "-");
        let id = format!("{ts}-somehost");
        assert!(!id.contains(':'));
    }
}
