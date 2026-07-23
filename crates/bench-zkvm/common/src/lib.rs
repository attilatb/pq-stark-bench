//! Shared results schema for the in-circuit (zkVM) benchmarks.
//!
//! The schema extends the base results file from KICKOFF.md section 6 with the
//! fields a cross-prover comparison actually needs: which prover and version,
//! the guest instruction set, the proving backend and mode, the claimed
//! security level, the batch shape, and the measured cost.
//!
//! Two rules from the build plan are enforced structurally here:
//!   1. A cycle count is always tagged with its prover, because RISC Zero and
//!      SP1 cycles are different units and must never share a chart axis.
//!   2. Every result carries its caveats inline, so a chart can always render
//!      the precompile and security disclosures next to the number.

#![forbid(unsafe_code)]

use pqb_core::env::Environment;
use serde::{Deserialize, Serialize};

/// Which family a scheme belongs to. Mirrors the native harness.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Classical,
    Lattice,
    Hash,
}

/// The scheme under measurement and exactly how it was built.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scheme {
    pub name: String,
    pub family: Family,
    /// e.g. "RFC 8032", "FIPS 204", "NIST selected (FIPS 206 not published)".
    pub spec: String,
    pub crate_name: String,
    pub crate_version: String,
    /// The hash primitive on the critical path, e.g. "SHA-512", "SHAKE-256".
    pub hash_primitive: String,
    /// Whether this build is standards conformant (as opposed to substituting a
    /// prover-friendly hash like Poseidon or RPO).
    pub conformant: bool,
    /// True if the implementation crate is a prerelease.
    pub prerelease: bool,
}

/// How a security level figure was obtained.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityKind {
    /// The prover documents this as a proven bound.
    Proven,
    /// The prover documents this as a conjectured or claimed level.
    Claimed,
    /// We could not find a documented figure.
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityBits {
    pub value: Option<u32>,
    pub kind: SecurityKind,
    /// URL documenting the figure, with an access date in the text.
    pub source: String,
}

/// The prover and precisely how it was configured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prover {
    /// "risc0" or "sp1". This is the axis key: cycles never cross it.
    pub name: String,
    pub version: String,
    /// Guest instruction set, e.g. "riscv32im" (risc0) or "riscv64im" (sp1).
    pub isa: String,
    /// "cpu-scalar", "metal", "cuda", "avx512".
    pub backend: String,
    /// "execute", "core", "composite", "succinct", "compressed", "groth16".
    pub proof_mode: String,
    pub security_bits: SecurityBits,
    /// RISC Zero segment limit (power of two). None for other provers.
    pub segment_limit_po2: Option<u32>,
    /// Precompiles the guest actually invoked, by name.
    pub precompiles_used: Vec<String>,
    /// True when an accelerated build was asserted to actually use its
    /// precompile. Meaningless (and false) for a stock build.
    pub precompile_assert_passed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topology {
    /// A single guest program loops over all N signatures.
    Flat,
    /// Signatures are proved in chunks and recursively aggregated.
    Recursive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub n: u32,
    pub topology: Topology,
    /// Recursion arity, when topology is recursive.
    pub arity: Option<u32>,
}

/// Measured cost of one workload. Fields are null when not measured on this
/// machine rather than guessed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    /// User cycles, tagged by prover.name. Deterministic and machine
    /// independent, which is why it is the CI regression signal.
    pub cycles: Option<u64>,
    /// How the cycle count was obtained, e.g. "SessionInfo.cycles".
    pub cycles_source: Option<String>,
    /// Total cycles including power-of-two padding, when the prover exposes it.
    pub total_cycles: Option<u64>,
    /// Prover wall-clock in milliseconds. Hardware dependent: read with
    /// environment.hardware_class, never mixed across classes.
    pub prove_ms: Option<f64>,
    /// Peak resident set size of the prover process, in megabytes.
    pub peak_ram_mb: Option<f64>,
    pub proof_bytes: Option<usize>,
    /// Verifier wall-clock in milliseconds.
    pub verify_ms: Option<f64>,
}

/// Status of a workload. Aborted workloads are published, not dropped.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    pub scheme: Scheme,
    pub prover: Prover,
    pub batch: Batch,
    pub cost: Cost,
    pub status: Status,
    /// Set when status is Aborted, e.g. "peak_rss_exceeded", "wall_timeout".
    pub abort_reason: Option<String>,
    /// Disclosures that must render next to any chart of this workload.
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkvmResultsFile {
    pub run_id: String,
    pub kind: String,
    pub schema_version: u32,
    pub generated_at: String,
    pub environment: Environment,
    /// Verbatim tool versions, e.g. {"cargo-risczero": "3.0.6", "r0vm": "3.0.6"}.
    pub toolchain: std::collections::BTreeMap<String, String>,
    pub workloads: Vec<Workload>,
}

impl ZkvmResultsFile {
    /// Serialize to pretty JSON with a trailing newline.
    pub fn to_json(&self) -> String {
        let mut s = serde_json::to_string_pretty(self).expect("results serialize");
        s.push('\n');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let wl = Workload {
            scheme: Scheme {
                name: "ed25519".into(),
                family: Family::Classical,
                spec: "RFC 8032".into(),
                crate_name: "ed25519-dalek".into(),
                crate_version: "2.1".into(),
                hash_primitive: "SHA-512".into(),
                conformant: true,
                prerelease: false,
            },
            prover: Prover {
                name: "risc0".into(),
                version: "3.0.6".into(),
                isa: "riscv32im".into(),
                backend: "metal".into(),
                proof_mode: "succinct".into(),
                security_bits: SecurityBits {
                    value: Some(96),
                    kind: SecurityKind::Claimed,
                    source: "docs, accessed 2026-07-23".into(),
                },
                segment_limit_po2: Some(20),
                precompiles_used: vec![],
                precompile_assert_passed: false,
            },
            batch: Batch {
                n: 1,
                topology: Topology::Flat,
                arity: None,
            },
            cost: Cost {
                cycles: Some(123456),
                cycles_source: Some("SessionInfo.cycles".into()),
                total_cycles: Some(1 << 20),
                prove_ms: None,
                peak_ram_mb: None,
                proof_bytes: None,
                verify_ms: None,
            },
            status: Status::Ok,
            abort_reason: None,
            caveats: vec!["no lattice precompile on this prover".into()],
        };
        let json = serde_json::to_string(&wl).unwrap();
        let back: Workload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.batch.n, 1);
        assert_eq!(back.cost.cycles, Some(123456));
    }
}
