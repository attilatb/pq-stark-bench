//! Signature schemes under measurement.
//!
//! Every scheme binds to a vetted third-party implementation. No cryptography
//! is implemented in this repository, per the project's hard rules.
//!
//! Each benchmark measures three operations against the canonical transaction
//! signing preimage from `tx-format`, so the message being signed is identical
//! across schemes and the comparison is like-for-like.

use crate::stats::{measure, Timing};
use serde::Serialize;

/// Which family a scheme belongs to. Used for grouping on the dashboard.
///
/// `Lattice` and `Hash` are unconstructed until the post-quantum schemes land
/// in Phase 1b. They are declared now because the dashboard's colour mapping
/// and the results schema are already keyed on them.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum Family {
    Classical,
    Lattice,
    Hash,
}

/// Measured results for one scheme.
#[derive(Debug, Clone, Serialize)]
pub struct SchemeMeasurement {
    pub scheme: String,
    pub family: Family,
    /// Standardization status, stated precisely. Never overstated: Falcon has
    /// no published FIPS draft as of 2026-07-23, so it is not called FIPS 206.
    pub standard: String,
    /// Which crate produced these numbers, so a reader can reproduce them.
    pub implementation: String,
    pub pubkey_bytes: usize,
    pub sig_bytes: usize,
    pub keygen: Timing,
    pub sign: Timing,
    pub verify: Timing,
}

/// Iteration counts. The brief requires N >= 100; keygen for some schemes is
/// slow enough that we keep it separately tunable.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub warmup: usize,
    pub iterations: usize,
    pub keygen_iterations: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            warmup: 20,
            iterations: 200,
            keygen_iterations: 100,
        }
    }
}

/// The canonical message every scheme signs.
///
/// Built from the transaction signing preimage so that the measured cost
/// corresponds to a real transaction rather than an arbitrary blob.
pub fn canonical_message(pubkey: &[u8]) -> Vec<u8> {
    tx_format::signing_preimage(
        1,           // version
        42,          // nonce
        pubkey,      // from_pubkey
        &[0xAB; 32], // to_address
        1_000_000,   // amount
        1_000,       // fee
        &[0u8; 32],  // payload_hash, zeroed
    )
}

// ---------------------------------------------------------------------------
// Ed25519 (classical baseline)
// ---------------------------------------------------------------------------

pub fn bench_ed25519(budget: Budget) -> SchemeMeasurement {
    use ed25519_dalek::rand_core::UnwrapErr;
    use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};

    let mut rng = UnwrapErr(getrandom::SysRng);

    // One fixed keypair for the sign/verify measurements.
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let pubkey_bytes = verifying_key.to_bytes();
    let msg = canonical_message(&pubkey_bytes);

    let signature: Signature = signing_key.sign(&msg);

    // Correctness gate: never benchmark a path that does not actually verify.
    assert!(
        verifying_key.verify(&msg, &signature).is_ok(),
        "ed25519 signature must verify before benchmarking"
    );

    let keygen = measure(budget.warmup, budget.keygen_iterations, || {
        SigningKey::generate(&mut UnwrapErr(getrandom::SysRng))
    });

    let sign = measure(budget.warmup, budget.iterations, || signing_key.sign(&msg));

    let verify = measure(budget.warmup, budget.iterations, || {
        verifying_key.verify(&msg, &signature).is_ok()
    });

    SchemeMeasurement {
        scheme: "ed25519".to_string(),
        family: Family::Classical,
        standard: "RFC 8032 (classical, not post-quantum)".to_string(),
        implementation: "ed25519-dalek".to_string(),
        pubkey_bytes: pubkey_bytes.len(),
        sig_bytes: signature.to_bytes().len(),
        keygen,
        sign,
        verify,
    }
}

// ---------------------------------------------------------------------------
// ECDSA over secp256k1 (classical baseline, the Bitcoin/Ethereum curve)
// ---------------------------------------------------------------------------

pub fn bench_ecdsa_secp256k1(budget: Budget) -> SchemeMeasurement {
    use k256::ecdsa::signature::{Signer, Verifier};
    use k256::ecdsa::{Signature, SigningKey};
    use k256::elliptic_curve::rand_core::UnwrapErr;
    use k256::elliptic_curve::Generate;

    let mut rng = UnwrapErr(getrandom::SysRng);

    let signing_key = SigningKey::generate_from_rng(&mut rng);
    let verifying_key = *signing_key.verifying_key();
    // SEC1 compressed encoding, which is what chains actually store.
    let pubkey_encoded = verifying_key.to_sec1_point(true);
    let pubkey_bytes = pubkey_encoded.as_bytes().to_vec();

    let msg = canonical_message(&pubkey_bytes);
    let signature: Signature = signing_key.sign(&msg);

    assert!(
        verifying_key.verify(&msg, &signature).is_ok(),
        "ecdsa secp256k1 signature must verify before benchmarking"
    );

    let keygen = measure(budget.warmup, budget.keygen_iterations, || {
        SigningKey::generate_from_rng(&mut UnwrapErr(getrandom::SysRng))
    });

    let sign = measure(budget.warmup, budget.iterations, || {
        let s: Signature = signing_key.sign(&msg);
        s
    });

    let verify = measure(budget.warmup, budget.iterations, || {
        verifying_key.verify(&msg, &signature).is_ok()
    });

    SchemeMeasurement {
        scheme: "ecdsa-secp256k1".to_string(),
        family: Family::Classical,
        standard: "SEC1 / FIPS 186-5 curve secp256k1 (classical, not post-quantum)".to_string(),
        implementation: "k256".to_string(),
        pubkey_bytes: pubkey_bytes.len(),
        sig_bytes: signature.to_bytes().len(),
        keygen,
        sign,
        verify,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Budget {
        Budget {
            warmup: 1,
            iterations: 3,
            keygen_iterations: 3,
        }
    }

    #[test]
    fn ed25519_sizes_match_the_specification() {
        let m = bench_ed25519(tiny());
        assert_eq!(m.pubkey_bytes, 32, "Ed25519 public key is 32 bytes");
        assert_eq!(m.sig_bytes, 64, "Ed25519 signature is 64 bytes");
        assert!(m.verify.median_ns > 0);
    }

    #[test]
    fn ecdsa_sizes_are_sane() {
        let m = bench_ecdsa_secp256k1(tiny());
        assert_eq!(
            m.pubkey_bytes, 33,
            "compressed secp256k1 public key is 33 bytes"
        );
        assert_eq!(m.sig_bytes, 64, "compact ECDSA signature is 64 bytes");
        assert!(m.verify.median_ns > 0);
    }

    #[test]
    fn canonical_message_is_stable_and_correct_length() {
        let pk = [1u8; 32];
        let a = canonical_message(&pk);
        let b = canonical_message(&pk);
        assert_eq!(a, b, "message construction must be deterministic");
        assert_eq!(a.len(), tx_format::FIXED_FIELD_BYTES + 32);
    }
}
