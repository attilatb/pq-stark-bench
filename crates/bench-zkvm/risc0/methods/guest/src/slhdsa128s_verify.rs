//! RISC Zero guest: verify a batch of SLH-DSA-SHA2-128s (FIPS 205) signatures.
//!
//! Hash-based verification, dominated by SHA-256. Stock: no SHA-256 precompile
//! is routed to fips205 here, so it runs as plain RISC-V. Disclosed in results.

use fips205::slh_dsa_sha2_128s;
use fips205::traits::{SerDes, Verifier};
use risc0_zkvm::guest::env;

#[derive(serde::Serialize, serde::Deserialize)]
struct VerifyJob {
    message: Vec<u8>,
    /// SLH-DSA-128s public key, 32 bytes.
    pubkey: Vec<u8>,
    /// SLH-DSA-SHA2-128s signature, 7856 bytes.
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

    env::commit(&n);
    env::commit(&verified);
}
