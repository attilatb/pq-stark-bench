//! RISC Zero guest: verify a batch of Ed25519 signatures, accelerated.
//!
//! Identical verification logic to the stock ed25519_verify guest, but the
//! curve25519-dalek dependency is patched to the RISC Zero fork so field
//! arithmetic runs on the accelerator. The cycle count here versus the stock
//! guest measures exactly what the precompile is worth.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use risc0_zkvm::guest::env;

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

fn main() {
    let batch: Batch = env::read();

    let n = batch.jobs.len() as u32;
    let mut verified: u32 = 0;

    for job in &batch.jobs {
        let pk: [u8; 32] = match job.pubkey.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => continue,
        };
        let sig_bytes: [u8; 64] = match job.signature.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => continue,
        };
        if let Ok(vk) = VerifyingKey::from_bytes(&pk) {
            let sig = Signature::from_bytes(&sig_bytes);
            if vk.verify(&job.message, &sig).is_ok() {
                verified += 1;
            }
        }
    }

    env::commit(&n);
    env::commit(&verified);
}
