//! Minimal, dependency-free UTC timestamp formatting.
//!
//! Used to build the `run_id` and `generated_at` fields of every results file.
//! Kept dependency-free so the same code runs in the native harness and in the
//! zkVM host binaries without pulling a date crate into either.

use std::time::{SystemTime, UNIX_EPOCH};

/// RFC 3339 UTC timestamp, seconds precision, e.g. `2026-07-23T16-44-06Z`
/// once colons are replaced for filesystem safety by the caller.
pub fn rfc3339_utc(now: SystemTime) -> String {
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
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
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

/// A run identifier of the form `<timestamp>-<host>`, filesystem safe.
pub fn run_id(now: SystemTime, host_label: &str) -> String {
    format!("{}-{}", rfc3339_utc(now).replace(':', "-"), host_label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
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
        let id = run_id(UNIX_EPOCH, "somehost");
        assert!(!id.contains(':'));
        assert!(id.ends_with("-somehost"));
    }
}
