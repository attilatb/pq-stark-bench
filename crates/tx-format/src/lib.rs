//! Canonical transaction format for PQ-STARK-BENCH.
//!
//! Defined once here and reused by every benchmark so that bytes-per-transaction
//! accounting is identical across schemes. See KICKOFF.md section 5.
//!
//! This crate deliberately contains no cryptography. It is a byte-accounting
//! model only: it answers "how many bytes does a transaction occupy under
//! model X", given the signature and public key sizes of a scheme.

#![forbid(unsafe_code)]

use serde::Serialize;

/// Size in bytes of each fixed field in the canonical transaction.
pub mod field_sizes {
    /// `version: u8`
    pub const VERSION: usize = 1;
    /// `nonce: u64`
    pub const NONCE: usize = 8;
    /// `to_address: [u8; 32]`
    pub const TO_ADDRESS: usize = 32;
    /// `amount: u64`
    pub const AMOUNT: usize = 8;
    /// `fee: u64`
    pub const FEE: usize = 8;
    /// `payload_hash: [u8; 32]`, zeroed when unused but always present.
    pub const PAYLOAD_HASH: usize = 32;
    /// A 32-byte hash-of-public-key address, used in place of an inline public key.
    pub const KEY_HASH_ADDRESS: usize = 32;
}

/// Bytes occupied by the scheme-independent fields common to every model.
///
/// version + nonce + to_address + amount + fee + payload_hash.
pub const FIXED_FIELD_BYTES: usize = field_sizes::VERSION
    + field_sizes::NONCE
    + field_sizes::TO_ADDRESS
    + field_sizes::AMOUNT
    + field_sizes::FEE
    + field_sizes::PAYLOAD_HASH;

/// How a transaction is encoded for on-chain publication.
///
/// The three models differ only in what is published on-chain, not in what is
/// signed. Every model signs the same canonical preimage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TxModel {
    /// (a) The full public key travels inside the transaction.
    ///
    /// Simplest, and the worst case for post-quantum schemes because public
    /// keys are large. Included because it is what a naive design does.
    PubkeyInTx,
    /// (b) A 32-byte key-hash address travels on-chain; the public key is
    /// revealed in witness data.
    ///
    /// This is what serious classical designs use. On-chain bytes exclude the
    /// public key; `with_witness` accounts for it separately.
    AddressOnly,
    /// (c) Validity-proof settlement: neither the signature nor the public key
    /// is ever published on-chain.
    ///
    /// Signatures are verified inside the proof circuit, and only the state
    /// transition data is posted. This is the model PQ-STARK-BENCH exists to
    /// evaluate, and the reason signature size stops dominating the cost.
    RollupWitnessOnly,
}

impl TxModel {
    pub const ALL: [TxModel; 3] = [
        TxModel::PubkeyInTx,
        TxModel::AddressOnly,
        TxModel::RollupWitnessOnly,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            TxModel::PubkeyInTx => "pubkey_in_tx",
            TxModel::AddressOnly => "address_only",
            TxModel::RollupWitnessOnly => "rollup_witness_only",
        }
    }
}

/// Byte accounting for one transaction under one model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TxSize {
    pub model: TxModel,
    /// Bytes that must be published to the settlement layer per transaction.
    pub on_chain_bytes: usize,
    /// Bytes that exist but need not be published (witness / off-chain data).
    pub witness_bytes: usize,
    /// on_chain_bytes + witness_bytes. The total the prover must handle.
    pub total_bytes: usize,
}

/// Compute per-transaction byte accounting for a scheme with the given key and
/// signature sizes, under every model.
///
/// `pubkey_bytes` and `sig_bytes` are scheme-dependent and come from the actual
/// keypair and signature produced by the benchmark, never from a constant table.
pub fn tx_sizes(pubkey_bytes: usize, sig_bytes: usize) -> Vec<TxSize> {
    TxModel::ALL
        .iter()
        .map(|&model| tx_size(model, pubkey_bytes, sig_bytes))
        .collect()
}

/// Compute per-transaction byte accounting under a single model.
pub fn tx_size(model: TxModel, pubkey_bytes: usize, sig_bytes: usize) -> TxSize {
    let (on_chain_bytes, witness_bytes) = match model {
        // Full public key and signature are published.
        TxModel::PubkeyInTx => (FIXED_FIELD_BYTES + pubkey_bytes + sig_bytes, 0),
        // Key-hash address and signature are published; public key is witness.
        TxModel::AddressOnly => (
            FIXED_FIELD_BYTES + field_sizes::KEY_HASH_ADDRESS + sig_bytes,
            pubkey_bytes,
        ),
        // Only the state transition is published. Signature and public key are
        // consumed by the prover and never posted.
        TxModel::RollupWitnessOnly => (
            FIXED_FIELD_BYTES + field_sizes::KEY_HASH_ADDRESS,
            pubkey_bytes + sig_bytes,
        ),
    };

    TxSize {
        model,
        on_chain_bytes,
        witness_bytes,
        total_bytes: on_chain_bytes + witness_bytes,
    }
}

/// The canonical signing preimage: every field except the signature itself.
///
/// Under all three models the same bytes are signed, so a signature produced
/// for one model is valid under the others. Encoding is fixed-width
/// little-endian to keep it unambiguous and trivially reproducible in a
/// zkVM guest.
pub fn signing_preimage(
    version: u8,
    nonce: u64,
    from_pubkey: &[u8],
    to_address: &[u8; 32],
    amount: u64,
    fee: u64,
    payload_hash: &[u8; 32],
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(FIXED_FIELD_BYTES + from_pubkey.len());
    buf.push(version);
    buf.extend_from_slice(&nonce.to_le_bytes());
    buf.extend_from_slice(from_pubkey);
    buf.extend_from_slice(to_address);
    buf.extend_from_slice(&amount.to_le_bytes());
    buf.extend_from_slice(&fee.to_le_bytes());
    buf.extend_from_slice(payload_hash);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_fields_are_89_bytes() {
        // 1 + 8 + 32 + 8 + 8 + 32
        assert_eq!(FIXED_FIELD_BYTES, 89);
    }

    #[test]
    fn ed25519_shaped_sizes() {
        // Ed25519: 32-byte public key, 64-byte signature.
        let sizes = tx_sizes(32, 64);
        let by = |m: TxModel| *sizes.iter().find(|s| s.model == m).unwrap();

        assert_eq!(by(TxModel::PubkeyInTx).on_chain_bytes, 89 + 32 + 64);
        assert_eq!(by(TxModel::PubkeyInTx).witness_bytes, 0);

        assert_eq!(by(TxModel::AddressOnly).on_chain_bytes, 89 + 32 + 64);
        assert_eq!(by(TxModel::AddressOnly).witness_bytes, 32);

        assert_eq!(by(TxModel::RollupWitnessOnly).on_chain_bytes, 89 + 32);
        assert_eq!(by(TxModel::RollupWitnessOnly).witness_bytes, 32 + 64);
    }

    #[test]
    fn rollup_on_chain_size_is_scheme_independent() {
        // The central claim of the project: under validity-proof settlement the
        // on-chain footprint does not depend on the signature scheme at all.
        let ed25519 = tx_size(TxModel::RollupWitnessOnly, 32, 64);
        let ml_dsa_44 = tx_size(TxModel::RollupWitnessOnly, 1312, 2420);
        let slh_dsa_128f = tx_size(TxModel::RollupWitnessOnly, 32, 17088);

        assert_eq!(ed25519.on_chain_bytes, ml_dsa_44.on_chain_bytes);
        assert_eq!(ed25519.on_chain_bytes, slh_dsa_128f.on_chain_bytes);

        // But the prover still has to consume the bytes.
        assert!(slh_dsa_128f.total_bytes > ed25519.total_bytes);
    }

    #[test]
    fn totals_are_consistent() {
        for s in tx_sizes(1312, 2420) {
            assert_eq!(s.total_bytes, s.on_chain_bytes + s.witness_bytes);
        }
    }

    #[test]
    fn preimage_length_matches_fixed_fields_plus_pubkey() {
        let pk = [7u8; 32];
        let p = signing_preimage(1, 42, &pk, &[9u8; 32], 1000, 10, &[0u8; 32]);
        assert_eq!(p.len(), FIXED_FIELD_BYTES + pk.len());
    }
}
