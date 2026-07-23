//! Shared core for PQ-STARK-BENCH harnesses.
//!
//! Holds the pieces every benchmark needs regardless of whether it runs
//! natively or drives a zkVM prover: environment capture (including the
//! `hardware_class` that keeps runs from different machines off the same
//! chart), the timing harness, and dependency-free timestamp formatting.
//!
//! No cryptography lives here, and none ever should.

#![forbid(unsafe_code)]

pub mod env;
pub mod stats;
pub mod time;

pub use env::{capture, host_label, Environment};
pub use stats::{measure_bounded, Timing};
pub use time::{rfc3339_utc, run_id};
