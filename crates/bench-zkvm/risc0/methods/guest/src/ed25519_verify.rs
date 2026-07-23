//! RISC Zero guest: verify a batch of Ed25519 signatures.
//!
//! Reads N (message, public key, signature) tuples from the host, verifies each
//! one inside the zkVM, and commits the batch size and the number that verified
//! to the journal. The host asserts that every signature verified, so a proof
//! attests "these N Ed25519 signatures are all valid".
//!
//! This is the classical control column. The post-quantum guests share the same
//! shape so the comparison is like for like.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use risc0_zkvm::guest::env;

/// One signature verification job. Byte vectors rather than fixed arrays,
/// because serde's derive only covers arrays up to length 32 and the signature
/// is 64 bytes. Lengths are checked at use.
#[derive(serde::Serialize, serde::Deserialize)]
struct VerifyJob {
    /// The exact bytes that were signed (the canonical transaction preimage).
    message: Vec<u8>,
    /// Ed25519 public key, 32 bytes.
    pubkey: Vec<u8>,
    /// Ed25519 signature, 64 bytes.
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
        let pk: [u8; 32] = job.pubkey.as_slice().try_into().expect("pubkey is 32 bytes");
        let sig_bytes: [u8; 64] = job
            .signature
            .as_slice()
            .try_into()
            .expect("signature is 64 bytes");
        let vk = VerifyingKey::from_bytes(&pk).expect("valid public key");
        let sig = Signature::from_bytes(&sig_bytes);
        if vk.verify(&job.message, &sig).is_ok() {
            verified += 1;
        }
    }

    // Commit the batch size and how many verified. The host checks verified == n.
    env::commit(&n);
    env::commit(&verified);
}
