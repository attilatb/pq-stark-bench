//! SP1 guest: verify a batch of ML-DSA-44 (FIPS 204) signatures.
//!
//! Stock build: no SHAKE precompile reaches fips204, so SHAKE-256 runs as plain
//! RISC-V. Disclosed in the results file.

#![no_main]
sp1_zkvm::entrypoint!(main);

use fips204::ml_dsa_44;
use fips204::traits::{SerDes, Verifier};

#[derive(serde::Serialize, serde::Deserialize)]
struct VerifyJob {
    message: Vec<u8>,
    pubkey: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Batch {
    jobs: Vec<VerifyJob>,
}

pub fn main() {
    let batch = sp1_zkvm::io::read::<Batch>();

    let n = batch.jobs.len() as u32;
    let mut verified: u32 = 0;
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

    sp1_zkvm::io::commit(&n);
    sp1_zkvm::io::commit(&verified);
}
