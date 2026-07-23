//! Timing harness and summary statistics.
//!
//! Deliberately simple and inspectable: we collect raw per-iteration
//! nanosecond samples and derive statistics from them, rather than reporting a
//! single averaged number. Median and p95 come from the sorted sample vector
//! using the nearest-rank method, which is what SUPERCOP-style benchmarking
//! conventions expect and which does not interpolate values that were never
//! measured.

use serde::Serialize;
use std::hint::black_box;
use std::time::Instant;

/// Summary of one timed measurement.
#[derive(Debug, Clone, Serialize)]
pub struct Timing {
    pub median_ns: u64,
    pub p95_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub mean_ns: u64,
    /// Iterations actually measured, excluding warmup.
    pub iterations: usize,
    /// Warmup iterations discarded before measurement began.
    pub warmup_iterations: usize,
}

/// Nearest-rank percentile over an already-sorted slice.
///
/// `p` is in (0, 1]. Returns the smallest value at or above the p-th
/// percentile. No interpolation: every reported figure is a value that was
/// actually observed.
fn percentile_sorted(sorted: &[u64], p: f64) -> u64 {
    debug_assert!(!sorted.is_empty());
    debug_assert!(p > 0.0 && p <= 1.0);
    let n = sorted.len() as f64;
    let rank = (p * n).ceil() as usize;
    let idx = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[idx]
}

/// Time a closure, discarding `warmup` iterations then measuring `iterations`.
///
/// The closure's return value is passed through `black_box` so the optimizer
/// cannot delete the work being measured.
pub fn measure<T, F>(warmup: usize, iterations: usize, mut f: F) -> Timing
where
    F: FnMut() -> T,
{
    assert!(iterations > 0, "iterations must be non-zero");

    for _ in 0..warmup {
        black_box(f());
    }

    let mut samples: Vec<u64> = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let out = f();
        let elapsed = start.elapsed();
        black_box(out);
        samples.push(elapsed.as_nanos() as u64);
    }

    summarize(&mut samples, warmup)
}

fn summarize(samples: &mut [u64], warmup: usize) -> Timing {
    samples.sort_unstable();
    let n = samples.len();
    let sum: u128 = samples.iter().map(|&v| v as u128).sum();

    let median_ns = if n % 2 == 1 {
        samples[n / 2]
    } else {
        // Average the two central samples. Both were observed.
        ((samples[n / 2 - 1] as u128 + samples[n / 2] as u128) / 2) as u64
    };

    Timing {
        median_ns,
        p95_ns: percentile_sorted(samples, 0.95),
        min_ns: samples[0],
        max_ns: samples[n - 1],
        mean_ns: (sum / n as u128) as u64,
        iterations: n,
        warmup_iterations: warmup,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_nearest_rank_is_an_observed_value() {
        let s: Vec<u64> = (1..=100).collect();
        // 95th percentile of 1..=100 by nearest rank is the 95th element.
        assert_eq!(percentile_sorted(&s, 0.95), 95);
        assert_eq!(percentile_sorted(&s, 1.0), 100);
    }

    #[test]
    fn percentile_handles_single_sample() {
        assert_eq!(percentile_sorted(&[42], 0.95), 42);
    }

    #[test]
    fn median_odd_and_even() {
        let mut odd = vec![3u64, 1, 2];
        assert_eq!(summarize(&mut odd, 0).median_ns, 2);

        let mut even = vec![4u64, 1, 2, 3];
        assert_eq!(summarize(&mut even, 0).median_ns, 2); // (2+3)/2 = 2 (integer)
    }

    #[test]
    fn summarize_reports_bounds_and_count() {
        let mut s = vec![10u64, 20, 30, 40, 50];
        let t = summarize(&mut s, 7);
        assert_eq!(t.min_ns, 10);
        assert_eq!(t.max_ns, 50);
        assert_eq!(t.iterations, 5);
        assert_eq!(t.warmup_iterations, 7);
        assert_eq!(t.mean_ns, 30);
    }

    #[test]
    fn measure_runs_the_requested_number_of_iterations() {
        let mut calls = 0usize;
        let t = measure(3, 10, || {
            calls += 1;
            calls
        });
        assert_eq!(t.iterations, 10);
        assert_eq!(calls, 13, "warmup plus measured iterations");
    }
}
