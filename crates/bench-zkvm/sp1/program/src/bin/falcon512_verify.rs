//! SP1 guest: verify a batch of Falcon-512 (FN-DSA) signatures.
//!
//! Integer-only verification via fn-dsa-vrfy, safe in the soft-float guest.

#![no_main]
sp1_zkvm::entrypoint!(main);

use fn_dsa_vrfy::{VerifyingKey, VerifyingKeyStandard, DOMAIN_NONE, HASH_ID_RAW};

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
        if let Some(vk) = VerifyingKeyStandard::decode(&job.pubkey) {
            if vk.verify(&job.signature, &DOMAIN_NONE, &HASH_ID_RAW, &job.message) {
                verified += 1;
            }
        }
    }

    sp1_zkvm::io::commit(&n);
    sp1_zkvm::io::commit(&verified);
}
