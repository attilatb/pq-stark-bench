// Capture the exact rustc that compiled this benchmark, so every results file
// records the compiler rather than assuming it. Reproducibility depends on this
// being the real value, not a hardcoded string.

use std::process::Command;

fn main() {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=PQB_RUSTC_VERSION={version}");
    println!(
        "cargo:rustc-env=PQB_TARGET={}",
        std::env::var("TARGET").unwrap_or_default()
    );
    println!("cargo:rerun-if-changed=build.rs");
}
