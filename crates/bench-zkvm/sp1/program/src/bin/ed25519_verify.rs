//! SP1 guest: verify a batch of Ed25519 signatures.
//!
//! Stock ed25519-dalek, no SP1 curve25519 precompile. The wire format matches
//! the RISC Zero guest so the two provers verify identical inputs.

#![no_main]
sp1_zkvm::entrypoint!(main);

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

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

    sp1_zkvm::io::commit(&n);
    sp1_zkvm::io::commit(&verified);
}
