//! RISC Zero guest: verify a batch of ML-DSA-44 (FIPS 204) signatures.
//!
//! Stock build: no SHAKE precompile reaches fips204, so the SHAKE-256 work runs
//! as plain RISC-V. That is disclosed in the results file. Verification is
//! deterministic and needs no RNG.

use fips204::ml_dsa_44;
// SerDes provides try_from_bytes; Verifier provides verify.
use fips204::traits::{SerDes, Verifier};
use risc0_zkvm::guest::env;

#[derive(serde::Serialize, serde::Deserialize)]
struct VerifyJob {
    message: Vec<u8>,
    /// ML-DSA-44 public key, 1312 bytes.
    pubkey: Vec<u8>,
    /// ML-DSA-44 signature, 2420 bytes.
    signature: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Batch {
    jobs: Vec<VerifyJob>,
}

fn main() {
    let batch: Batch = env::read();

    let n = batch.jobs.len() as u32;
    let mut verified: u32 = 0;

    // Empty FIPS 204 context string, matching the host.
    const CTX: &[u8] = b"";

    for job in &batch.jobs {
        let pk_bytes: [u8; ml_dsa_44::PK_LEN] = match job.pubkey.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => continue,
        };
        let sig_bytes: [u8; ml_dsa_44::SIG_LEN] = match job.signature.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => continue,
        };
        if let Ok(pk) = ml_dsa_44::PublicKey::try_from_bytes(pk_bytes) {
            if pk.verify(&job.message, &sig_bytes, CTX) {
                verified += 1;
            }
        }
    }

    env::commit(&n);
    env::commit(&verified);
}
