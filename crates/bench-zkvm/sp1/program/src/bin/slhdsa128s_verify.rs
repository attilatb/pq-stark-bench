//! SP1 guest: verify a batch of SLH-DSA-SHA2-128s (FIPS 205) signatures.
//!
//! Hash-based verification, dominated by SHA-256. Stock: no SHA-256 precompile
//! routed to fips205 here. Disclosed in the results file.

#![no_main]
sp1_zkvm::entrypoint!(main);

use fips205::slh_dsa_sha2_128s;
use fips205::traits::{SerDes, Verifier};

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
        let pk_bytes: [u8; slh_dsa_sha2_128s::PK_LEN] = match job.pubkey.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => continue,
        };
        let sig_bytes: [u8; slh_dsa_sha2_128s::SIG_LEN] = match job.signature.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => continue,
        };
        // fips205 takes the byte array by reference, unlike fips204.
        if let Ok(pk) = slh_dsa_sha2_128s::PublicKey::try_from_bytes(&pk_bytes) {
            if pk.verify(&job.message, &sig_bytes, CTX) {
                verified += 1;
            }
        }
    }

    sp1_zkvm::io::commit(&n);
    sp1_zkvm::io::commit(&verified);
}
