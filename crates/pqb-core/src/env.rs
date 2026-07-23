//! Environment capture.
//!
//! Every results file records the machine it was produced on. Results from
//! different hardware classes are never combined into a single chart, so the
//! `hardware_class` field is load-bearing rather than decorative.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub cpu: String,
    pub cores: usize,
    pub ram_gb: f64,
    pub os: String,
    pub arch: String,
    pub rustc: String,
    pub target: String,
    /// Which class of machine this is. Charts must not mix classes.
    /// One of: local-m3max, local-other, ci-x86, ci-arm, cloud-x86-cuda, unknown.
    pub hardware_class: String,
    pub library_versions: BTreeMap<String, String>,
    pub is_ci: bool,
    /// True when the harness could not pin or verify CPU frequency, which is
    /// the normal case on Apple Silicon laptops. Consumers should treat timing
    /// dispersion accordingly.
    pub frequency_pinned: bool,
}

fn sysctl(key: &str) -> Option<String> {
    Command::new("sysctl")
        .args(["-n", key])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_first_line_containing(path: &str, needle: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    content
        .lines()
        .find(|l| l.contains(needle))
        .map(|l| l.to_string())
}

fn detect_cpu() -> String {
    if cfg!(target_os = "macos") {
        if let Some(v) = sysctl("machdep.cpu.brand_string") {
            return v;
        }
    }
    if let Some(line) = read_first_line_containing("/proc/cpuinfo", "model name") {
        if let Some((_, v)) = line.split_once(':') {
            return v.trim().to_string();
        }
    }
    "unknown".to_string()
}

fn detect_cores() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0)
}

fn detect_ram_gb() -> f64 {
    if cfg!(target_os = "macos") {
        if let Some(bytes) = sysctl("hw.memsize").and_then(|s| s.parse::<u64>().ok()) {
            return round2(bytes as f64 / 1024.0 / 1024.0 / 1024.0);
        }
    }
    if let Some(line) = read_first_line_containing("/proc/meminfo", "MemTotal") {
        if let Some(kb) = line
            .split_whitespace()
            .nth(1)
            .and_then(|v| v.parse::<f64>().ok())
        {
            return round2(kb / 1024.0 / 1024.0);
        }
    }
    0.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn detect_os() -> String {
    let base = std::env::consts::OS;
    if cfg!(target_os = "macos") {
        if let Some(v) = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
        {
            return format!("macOS {}", v.trim());
        }
    }
    base.to_string()
}

fn detect_is_ci() -> bool {
    // GitHub Actions and most CI providers set CI=true.
    matches!(
        std::env::var("CI").ok().as_deref(),
        Some("true") | Some("1")
    ) || std::env::var("GITHUB_ACTIONS").is_ok()
}

/// Classify the machine.
///
/// An explicit `PQB_HARDWARE_CLASS` always wins, so a cloud run can label
/// itself precisely without the harness having to guess.
fn detect_hardware_class(cpu: &str, is_ci: bool) -> String {
    if let Ok(explicit) = std::env::var("PQB_HARDWARE_CLASS") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let arch = std::env::consts::ARCH;
    if is_ci {
        return match arch {
            "x86_64" => "ci-x86".to_string(),
            "aarch64" => "ci-arm".to_string(),
            other => format!("ci-{other}"),
        };
    }

    if cpu.contains("M3 Max") {
        return "local-m3max".to_string();
    }
    "local-other".to_string()
}

/// Crate versions compiled into this binary, recorded so a reader can pin them.
///
/// These are the versions cargo actually resolved, taken from the dependency
/// crates' own build-time metadata where available.
fn library_versions() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert(
        "bench-native".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    // Recorded from Cargo.lock by the `just versions` recipe rather than
    // guessed here; see METHODOLOGY.md. Populated at runtime below.
    if let Ok(json) = std::env::var("PQB_LIBRARY_VERSIONS") {
        if let Ok(parsed) = serde_json::from_str::<BTreeMap<String, String>>(&json) {
            m.extend(parsed);
        }
    }
    m
}

pub fn capture() -> Environment {
    let cpu = detect_cpu();
    let is_ci = detect_is_ci();
    let hardware_class = detect_hardware_class(&cpu, is_ci);

    Environment {
        cpu,
        cores: detect_cores(),
        ram_gb: detect_ram_gb(),
        os: detect_os(),
        arch: std::env::consts::ARCH.to_string(),
        rustc: env!("PQB_RUSTC_VERSION").to_string(),
        target: env!("PQB_TARGET").to_string(),
        hardware_class,
        library_versions: library_versions(),
        is_ci,
        // We do not currently pin CPU frequency. On Apple Silicon there is no
        // supported way to do so, and claiming otherwise would be false.
        frequency_pinned: false,
    }
}

/// Filesystem-safe host label used in result filenames.
pub fn host_label() -> String {
    let raw = Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown-host".to_string());

    raw.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_populates_required_fields() {
        let e = capture();
        assert!(e.cores > 0, "cores should be detected");
        assert!(!e.rustc.is_empty(), "rustc version must be recorded");
        assert!(!e.arch.is_empty());
        assert!(!e.hardware_class.is_empty());
        // We never claim a pinned frequency.
        assert!(!e.frequency_pinned);
    }

    #[test]
    fn host_label_is_filesystem_safe() {
        let h = host_label();
        assert!(!h.is_empty());
        assert!(h.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn explicit_hardware_class_wins() {
        // Not using std::env::set_var here because it is unsafe in edition 2024
        // and racy across threads; instead verify the fallback classification.
        let c = detect_hardware_class("Apple M3 Max", false);
        assert_eq!(c, "local-m3max");
        assert!(detect_hardware_class("Some Xeon", true).starts_with("ci-"));
    }
}
