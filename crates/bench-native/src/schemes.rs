//! Signature schemes under measurement.
//!
//! Every scheme binds to a vetted third-party implementation. No cryptography
//! is implemented in this repository, per the project's hard rules.
//!
//! Each benchmark measures three operations against the canonical transaction
//! signing preimage from `tx-format`, so the message being signed is identical
//! across schemes and the comparison is like-for-like.

use pqb_core::stats::{measure_bounded, Timing};
use serde::Serialize;
use std::time::Duration;

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

/// Iteration policy.
///
/// The project requires N >= 100 for any published figure, so `min_iterations`
/// is a hard floor. Above that floor we keep sampling up to `max_iterations`
/// only while a per-operation wall-clock budget lasts. SLH-DSA signing is
/// roughly four orders of magnitude slower than Ed25519 signing, so a single
/// fixed count would either undersample the fast schemes or make the slow ones
/// take many minutes.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub warmup: usize,
    pub min_iterations: usize,
    pub max_iterations: usize,
    pub per_op_budget: Duration,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            warmup: 10,
            min_iterations: 100,
            max_iterations: 1000,
            per_op_budget: Duration::from_secs(20),
        }
    }
}

impl Budget {
    /// Development pass. Not for publication: it drops below the N >= 100 floor.
    pub fn quick() -> Self {
        Self {
            warmup: 2,
            min_iterations: 10,
            max_iterations: 25,
            per_op_budget: Duration::from_secs(3),
        }
    }

    fn run<T, F: FnMut() -> T>(&self, f: F) -> Timing {
        measure_bounded(
            self.warmup,
            self.min_iterations,
            self.max_iterations,
            self.per_op_budget,
            f,
        )
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

    let keygen = budget.run(|| SigningKey::generate(&mut UnwrapErr(getrandom::SysRng)));

    let sign = budget.run(|| signing_key.sign(&msg));

    let verify = budget.run(|| verifying_key.verify(&msg, &signature).is_ok());

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

    let keygen = budget.run(|| SigningKey::generate_from_rng(&mut UnwrapErr(getrandom::SysRng)));

    let sign = budget.run(|| {
        let s: Signature = signing_key.sign(&msg);
        s
    });

    let verify = budget.run(|| verifying_key.verify(&msg, &signature).is_ok());

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

// ---------------------------------------------------------------------------
// ML-DSA-44 (FIPS 204, lattice)
// ---------------------------------------------------------------------------

pub fn bench_ml_dsa_44(budget: Budget) -> SchemeMeasurement {
    use fips204::ml_dsa_44;
    // try_keygen is a free function in the parameter-set module, so the
    // KeyGen trait itself is not needed in scope.
    use fips204::traits::{SerDes, Signer, Verifier};

    // FIPS 204 context string. Empty is the default and is what a chain would
    // use, so the measurement matches the transaction case.
    const CTX: &[u8] = b"";

    let (pk, sk) = ml_dsa_44::try_keygen().expect("ml-dsa-44 keygen");
    // into_bytes consumes the key, so serialize a clone and keep pk for verify.
    let pubkey_bytes = pk.clone().into_bytes();
    let msg = canonical_message(&pubkey_bytes);

    let sig = sk.try_sign(&msg, CTX).expect("ml-dsa-44 sign");
    assert!(
        pk.verify(&msg, &sig, CTX),
        "ml-dsa-44 signature must verify before benchmarking"
    );

    let keygen = budget.run(|| ml_dsa_44::try_keygen().expect("keygen"));
    let sign = budget.run(|| sk.try_sign(&msg, CTX).expect("sign"));
    let verify = budget.run(|| pk.verify(&msg, &sig, CTX));

    SchemeMeasurement {
        scheme: "ml-dsa-44".to_string(),
        family: Family::Lattice,
        standard: "NIST FIPS 204".to_string(),
        implementation: "fips204".to_string(),
        pubkey_bytes: ml_dsa_44::PK_LEN,
        sig_bytes: ml_dsa_44::SIG_LEN,
        keygen,
        sign,
        verify,
    }
}

// ---------------------------------------------------------------------------
// SLH-DSA (FIPS 205, hash based)
// ---------------------------------------------------------------------------
//
// FIPS 205 defines both SHA2 and SHAKE variants at every parameter set. The
// SHA2 variants are measured here and are named explicitly rather than as a
// bare "SLH-DSA-128s", because the two differ in cost and conflating them
// would be misleading.

macro_rules! bench_slh_dsa {
    ($fn_name:ident, $module:path, $label:expr) => {
        pub fn $fn_name(budget: Budget) -> SchemeMeasurement {
            use fips205::traits::{SerDes, Signer, Verifier};
            use $module as params;

            const CTX: &[u8] = b"";
            // Hedged signing (randomized) is the FIPS 205 default.
            const HEDGED: bool = true;

            let (pk, sk) = params::try_keygen().expect("slh-dsa keygen");
            // into_bytes consumes the key, so serialize a clone.
            let pubkey_bytes = pk.clone().into_bytes();
            let msg = canonical_message(&pubkey_bytes);

            let sig = sk.try_sign(&msg, CTX, HEDGED).expect("slh-dsa sign");
            assert!(
                pk.verify(&msg, &sig, CTX),
                "slh-dsa signature must verify before benchmarking"
            );

            let keygen = budget.run(|| params::try_keygen().expect("keygen"));
            let sign = budget.run(|| sk.try_sign(&msg, CTX, HEDGED).expect("sign"));
            let verify = budget.run(|| pk.verify(&msg, &sig, CTX));

            SchemeMeasurement {
                scheme: $label.to_string(),
                family: Family::Hash,
                standard: "NIST FIPS 205".to_string(),
                implementation: "fips205".to_string(),
                pubkey_bytes: params::PK_LEN,
                sig_bytes: params::SIG_LEN,
                keygen,
                sign,
                verify,
            }
        }
    };
}

bench_slh_dsa!(
    bench_slh_dsa_sha2_128s,
    fips205::slh_dsa_sha2_128s,
    "slh-dsa-sha2-128s"
);
bench_slh_dsa!(
    bench_slh_dsa_sha2_128f,
    fips205::slh_dsa_sha2_128f,
    "slh-dsa-sha2-128f"
);

// ---------------------------------------------------------------------------
// Falcon-512 / FN-DSA (NIST selected, lattice)
// ---------------------------------------------------------------------------
//
// Signing and key generation use floating-point Gaussian sampling. Verification
// is integer only, which is why the Phase 2 plan signs outside the guest and
// verifies inside it.
//
// FIPS 206 has no published draft as of 2026-07-23, so this is described as
// NIST selected rather than FIPS conformant.

pub fn bench_falcon_512(budget: Budget) -> SchemeMeasurement {
    use fn_dsa::{
        sign_key_size, signature_size, vrfy_key_size, KeyPairGenerator, KeyPairGeneratorStandard,
        SigningKey, SigningKeyStandard, VerifyingKey, VerifyingKeyStandard, DOMAIN_NONE,
        FN_DSA_LOGN_512, HASH_ID_RAW,
    };

    let logn = FN_DSA_LOGN_512;
    let mut rng = rand_core_06::OsRng;

    let mut kg = KeyPairGeneratorStandard::default();
    let mut sk_buf = vec![0u8; sign_key_size(logn)];
    let mut vk_buf = vec![0u8; vrfy_key_size(logn)];
    kg.keygen(logn, &mut rng, &mut sk_buf, &mut vk_buf);

    let msg = canonical_message(&vk_buf);

    let mut sk = SigningKeyStandard::decode(&sk_buf).expect("falcon signing key decode");
    let mut sig = vec![0u8; signature_size(logn)];
    sk.sign(&mut rng, &DOMAIN_NONE, &HASH_ID_RAW, &msg, &mut sig);

    let vk = VerifyingKeyStandard::decode(&vk_buf).expect("falcon verifying key decode");
    assert!(
        vk.verify(&sig, &DOMAIN_NONE, &HASH_ID_RAW, &msg),
        "falcon-512 signature must verify before benchmarking"
    );

    let keygen = budget.run(|| {
        let mut kg = KeyPairGeneratorStandard::default();
        let mut sk_b = vec![0u8; sign_key_size(logn)];
        let mut vk_b = vec![0u8; vrfy_key_size(logn)];
        kg.keygen(logn, &mut rand_core_06::OsRng, &mut sk_b, &mut vk_b);
        (sk_b, vk_b)
    });

    let sign = budget.run(|| {
        let mut s = vec![0u8; signature_size(logn)];
        sk.sign(
            &mut rand_core_06::OsRng,
            &DOMAIN_NONE,
            &HASH_ID_RAW,
            &msg,
            &mut s,
        );
        s
    });

    let verify = budget.run(|| vk.verify(&sig, &DOMAIN_NONE, &HASH_ID_RAW, &msg));

    SchemeMeasurement {
        scheme: "falcon-512".to_string(),
        family: Family::Lattice,
        standard: "NIST selected. FIPS 206 not published as of 2026-07-23.".to_string(),
        implementation: "fn-dsa".to_string(),
        pubkey_bytes: vk_buf.len(),
        sig_bytes: sig.len(),
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
            warmup: 0,
            min_iterations: 2,
            max_iterations: 2,
            per_op_budget: Duration::from_secs(30),
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
    fn ml_dsa_44_sizes_match_fips_204() {
        let m = bench_ml_dsa_44(tiny());
        assert_eq!(m.pubkey_bytes, 1312, "ML-DSA-44 public key is 1312 bytes");
        assert_eq!(m.sig_bytes, 2420, "ML-DSA-44 signature is 2420 bytes");
        assert!(m.verify.median_ns > 0);
    }

    #[test]
    fn slh_dsa_128s_sizes_match_fips_205() {
        let m = bench_slh_dsa_sha2_128s(tiny());
        assert_eq!(m.pubkey_bytes, 32, "SLH-DSA-128s public key is 32 bytes");
        assert_eq!(m.sig_bytes, 7856, "SLH-DSA-128s signature is 7856 bytes");
    }

    #[test]
    fn slh_dsa_128f_sizes_match_fips_205() {
        let m = bench_slh_dsa_sha2_128f(tiny());
        assert_eq!(m.pubkey_bytes, 32, "SLH-DSA-128f public key is 32 bytes");
        assert_eq!(m.sig_bytes, 17088, "SLH-DSA-128f signature is 17088 bytes");
    }

    #[test]
    fn falcon_512_sizes_match_the_specification() {
        let m = bench_falcon_512(tiny());
        assert_eq!(m.pubkey_bytes, 897, "Falcon-512 public key is 897 bytes");
        assert_eq!(m.sig_bytes, 666, "Falcon-512 signature is 666 bytes");
        assert!(m.verify.median_ns > 0);
    }

    #[test]
    fn falcon_is_never_described_as_fips_conformant() {
        // FIPS 206 has no published draft as of 2026-07-23, so claiming
        // conformance would be false. Mentioning FIPS 206 in order to say it is
        // NOT published is correct and must stay allowed, so this checks for
        // claims of conformance rather than for the string "FIPS 206".
        let m = bench_falcon_512(tiny());
        let s = m.standard.to_lowercase();
        for claim in [
            "fips 206 conformant",
            "fips 206 compliant",
            "fips 206 standard",
            "nist fips 206",
        ] {
            assert!(!s.contains(claim), "must not claim conformance: {claim}");
        }
        assert!(s.contains("nist selected"));
        assert!(s.contains("not published"));
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
