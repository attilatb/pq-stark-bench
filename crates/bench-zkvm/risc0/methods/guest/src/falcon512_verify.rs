//! RISC Zero guest: verify a batch of Falcon-512 (FN-DSA) signatures.
//!
//! Verification is integer only, so it runs in the soft-float guest without
//! pulling the floating-point signing path. Signing happens on the host.
//!
//! The wire format matches the other guests: N (message, pubkey, signature)
//! tuples in, batch size and verified count committed to the journal.

use fn_dsa_vrfy::{VerifyingKey, VerifyingKeyStandard, DOMAIN_NONE, HASH_ID_RAW};
use risc0_zkvm::guest::env;

#[derive(serde::Serialize, serde::Deserialize)]
struct VerifyJob {
    message: Vec<u8>,
    /// Falcon-512 verifying key, 897 bytes.
    pubkey: Vec<u8>,
    /// Falcon-512 signature, 666 bytes.
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
        if let Some(vk) = VerifyingKeyStandard::decode(&job.pubkey) {
            // Message is passed raw and hashed inside verify with HASH_ID_RAW,
            // matching how the host signed it.
            if vk.verify(&job.signature, &DOMAIN_NONE, &HASH_ID_RAW, &job.message) {
                verified += 1;
            }
        }
    }

    env::commit(&n);
    env::commit(&verified);
}
