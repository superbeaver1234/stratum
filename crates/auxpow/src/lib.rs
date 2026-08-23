#![forbid(unsafe_code)]

//! AuxPoW primitives. Consensus variants are adapter-owned; there is no
//! assumption that Namecoin/Dogecoin slot selection applies to every child.

use core::fmt;
use stratum_primitives::{sha256d, BitcoinBlockHeader, BlockHash, Hash256};

pub const ESF_CHAIN_ID: u32 = 1175;
pub const ESF_AUXPOW_ACTIVATION_HEIGHT: u32 = 31_733;
pub const ESF_MERGED_MINING_MAGIC: [u8; 4] = [0xfa, 0xbe, 0x6d, 0x6d];
pub const ESF_MAX_BRANCH_DEPTH: usize = 30;
pub const ESF_MAX_MERKLE_SIZE: u32 = 1 << 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuxPowError {
    InvalidMerkleSize(u32),
    BranchTooDeep { branch: &'static str, depth: usize },
    InvalidParentIndex(i32),
    InvalidChainIndex(i32),
    ChainIndexOutOfRange { index: u32, size: u32 },
    ChainBranchSizeMismatch { depth: usize, size: u32 },
    EmptyCoinbaseTransaction,
}

impl fmt::Display for AuxPowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMerkleSize(size) => write!(f, "invalid AuxPoW merkle size {size}"),
            Self::BranchTooDeep { branch, depth } => {
                write!(f, "{branch} merkle branch depth {depth} exceeds 30")
            }
            Self::InvalidParentIndex(index) => write!(f, "parent coinbase index must be 0, got {index}"),
            Self::InvalidChainIndex(index) => write!(f, "negative AuxPoW chain index {index}"),
            Self::ChainIndexOutOfRange { index, size } => {
                write!(f, "AuxPoW chain index {index} is outside merkle size {size}")
            }
            Self::ChainBranchSizeMismatch { depth, size } => write!(
                f,
                "AuxPoW chain branch depth {depth} does not match merkle size {size}"
            ),
            Self::EmptyCoinbaseTransaction => f.write_str("serialized parent coinbase transaction is empty"),
        }
    }
}

impl std::error::Error for AuxPowError {}

pub trait AuxChain {
    fn chain_id(&self) -> u32;

    fn slot_for(&self, merkle_nonce: u32, merkle_size: u32) -> Result<u32, AuxPowError>;

    fn encode_commitment(
        &self,
        aux_merkle_root: Hash256,
        merkle_size: u32,
        merkle_nonce: u32,
    ) -> Result<MergedMiningCommitment, AuxPowError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EsfAuxChain;

impl AuxChain for EsfAuxChain {
    fn chain_id(&self) -> u32 {
        ESF_CHAIN_ID
    }

    fn slot_for(&self, merkle_nonce: u32, merkle_size: u32) -> Result<u32, AuxPowError> {
        validate_merkle_size(merkle_size)?;
        Ok(merkle_nonce % merkle_size)
    }

    fn encode_commitment(
        &self,
        aux_merkle_root: Hash256,
        merkle_size: u32,
        merkle_nonce: u32,
    ) -> Result<MergedMiningCommitment, AuxPowError> {
        validate_merkle_size(merkle_size)?;
        let mut bytes = Vec::with_capacity(44);
        bytes.extend_from_slice(&ESF_MERGED_MINING_MAGIC);
        bytes.extend_from_slice(aux_merkle_root.as_wire_le_bytes());
        bytes.extend_from_slice(&merkle_size.to_le_bytes());
        bytes.extend_from_slice(&merkle_nonce.to_le_bytes());
        Ok(MergedMiningCommitment(bytes))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergedMiningCommitment(Vec<u8>);

impl MergedMiningCommitment {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// A complete Bitcoin-family transaction serialized exactly as it appears in
/// `CAuxPow::coinbaseTx` under ESF's `TX_WITH_WITNESS` stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedParentCoinbaseTx(Vec<u8>);

impl SerializedParentCoinbaseTx {
    pub fn new(bytes: Vec<u8>) -> Result<Self, AuxPowError> {
        if bytes.is_empty() {
            return Err(AuxPowError::EmptyCoinbaseTransaction);
        }
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EsfAuxPowProof {
    pub coinbase_tx: SerializedParentCoinbaseTx,
    pub hash_block: BlockHash,
    pub parent_merkle_branch: Vec<Hash256>,
    pub parent_index: i32,
    pub chain_merkle_branch: Vec<Hash256>,
    pub chain_index: i32,
    pub parent_header: BitcoinBlockHeader,
}

impl EsfAuxPowProof {
    pub fn new(
        coinbase_tx: SerializedParentCoinbaseTx,
        parent_merkle_branch: Vec<Hash256>,
        parent_index: i32,
        chain_merkle_branch: Vec<Hash256>,
        chain_index: i32,
        parent_header: BitcoinBlockHeader,
    ) -> Result<Self, AuxPowError> {
        if parent_merkle_branch.len() > ESF_MAX_BRANCH_DEPTH {
            return Err(AuxPowError::BranchTooDeep {
                branch: "parent",
                depth: parent_merkle_branch.len(),
            });
        }
        if chain_merkle_branch.len() > ESF_MAX_BRANCH_DEPTH {
            return Err(AuxPowError::BranchTooDeep {
                branch: "chain",
                depth: chain_merkle_branch.len(),
            });
        }
        if parent_index != 0 {
            return Err(AuxPowError::InvalidParentIndex(parent_index));
        }
        if chain_index < 0 {
            return Err(AuxPowError::InvalidChainIndex(chain_index));
        }
        Ok(Self {
            coinbase_tx,
            hash_block: parent_header.hash(),
            parent_merkle_branch,
            parent_index,
            chain_merkle_branch,
            chain_index,
            parent_header,
        })
    }

    /// Exact ESF `CAuxPow` serialization order on v29.1.0:
    /// transaction, parent hash, parent branch+index, chain branch+index,
    /// pure 80-byte parent header.
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(self.coinbase_tx.as_bytes());
        out.extend_from_slice(self.hash_block.as_wire_le_bytes());
        write_hash_vector(&mut out, &self.parent_merkle_branch);
        out.extend_from_slice(&self.parent_index.to_le_bytes());
        write_hash_vector(&mut out, &self.chain_merkle_branch);
        out.extend_from_slice(&self.chain_index.to_le_bytes());
        out.extend_from_slice(&self.parent_header.serialize_wire());
        out
    }

    pub fn validate_chain_shape(
        &self,
        merkle_size: u32,
        merkle_nonce: u32,
    ) -> Result<(), AuxPowError> {
        validate_merkle_size(merkle_size)?;
        let depth = merkle_size.trailing_zeros() as usize;
        if self.chain_merkle_branch.len() != depth {
            return Err(AuxPowError::ChainBranchSizeMismatch {
                depth: self.chain_merkle_branch.len(),
                size: merkle_size,
            });
        }
        let expected = EsfAuxChain.slot_for(merkle_nonce, merkle_size)?;
        let index = self.chain_index as u32;
        if index >= merkle_size {
            return Err(AuxPowError::ChainIndexOutOfRange {
                index,
                size: merkle_size,
            });
        }
        if index != expected {
            return Err(AuxPowError::ChainIndexOutOfRange {
                index,
                size: merkle_size,
            });
        }
        Ok(())
    }
}

pub fn apply_merkle_branch(
    mut hash: Hash256,
    branch: &[Hash256],
    mut index: u32,
) -> Hash256 {
    for sibling in branch {
        let mut pair = [0_u8; 64];
        if index & 1 == 1 {
            pair[..32].copy_from_slice(sibling.as_wire_le_bytes());
            pair[32..].copy_from_slice(hash.as_wire_le_bytes());
        } else {
            pair[..32].copy_from_slice(hash.as_wire_le_bytes());
            pair[32..].copy_from_slice(sibling.as_wire_le_bytes());
        }
        hash = sha256d(&pair);
        index >>= 1;
    }
    hash
}

fn validate_merkle_size(size: u32) -> Result<(), AuxPowError> {
    if size == 0 || !size.is_power_of_two() || size > ESF_MAX_MERKLE_SIZE {
        return Err(AuxPowError::InvalidMerkleSize(size));
    }
    Ok(())
}

fn write_hash_vector(out: &mut Vec<u8>, branch: &[Hash256]) {
    write_compact_size(out, branch.len() as u64);
    for hash in branch {
        out.extend_from_slice(hash.as_wire_le_bytes());
    }
}

fn write_compact_size(out: &mut Vec<u8>, value: u64) {
    match value {
        0..=0xfc => out.push(value as u8),
        0xfd..=0xffff => {
            out.push(0xfd);
            out.extend_from_slice(&(value as u16).to_le_bytes());
        }
        0x1_0000..=0xffff_ffff => {
            out.push(0xfe);
            out.extend_from_slice(&(value as u32).to_le_bytes());
        }
        _ => {
            out.push(0xff);
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stratum_primitives::CompactTarget;

    fn zero_parent_header() -> BitcoinBlockHeader {
        BitcoinBlockHeader {
            version: 1,
            previous_block: Hash256::ZERO,
            merkle_root: Hash256::ZERO,
            time: 1_000_000,
            bits: CompactTarget(0x207fffff),
            nonce: 0,
        }
    }

    #[test]
    fn esf_single_chain_commitment_matches_core_functional_test_layout() {
        let child = Hash256::from_display_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        )
        .unwrap();
        let commitment = EsfAuxChain.encode_commitment(child, 1, 0).unwrap();
        let bytes = commitment.as_bytes();
        assert_eq!(&bytes[..4], &ESF_MERGED_MINING_MAGIC);
        assert_eq!(&bytes[4..36], child.as_wire_le_bytes());
        assert_eq!(&bytes[36..40], &1_u32.to_le_bytes());
        assert_eq!(&bytes[40..44], &0_u32.to_le_bytes());
        assert_eq!(bytes.len(), 44);
    }

    #[test]
    fn esf_slot_is_nonce_mod_merkle_size_not_lcg() {
        assert_eq!(EsfAuxChain.slot_for(7, 4).unwrap(), 3);
        assert_eq!(EsfAuxChain.slot_for(8, 4).unwrap(), 0);
        assert!(EsfAuxChain.slot_for(0, 3).is_err());
    }

    #[test]
    fn auxpow_serialization_has_core_field_order() {
        let tx = SerializedParentCoinbaseTx::new(vec![0x01, 0x02, 0x03]).unwrap();
        let parent_header = zero_parent_header();
        let proof = EsfAuxPowProof::new(tx, vec![], 0, vec![], 0, parent_header).unwrap();
        let serialized = proof.serialize();

        assert_eq!(&serialized[..3], &[0x01, 0x02, 0x03]);
        assert_eq!(&serialized[3..35], proof.hash_block.as_wire_le_bytes());
        assert_eq!(serialized[35], 0); // empty parent branch CompactSize
        assert_eq!(&serialized[36..40], &0_i32.to_le_bytes());
        assert_eq!(serialized[40], 0); // empty chain branch CompactSize
        assert_eq!(&serialized[41..45], &0_i32.to_le_bytes());
        assert_eq!(&serialized[45..], &parent_header.serialize_wire());
    }

    #[test]
    fn hash_block_is_derived_from_parent_header() {
        let proof = EsfAuxPowProof::new(
            SerializedParentCoinbaseTx::new(vec![1]).unwrap(),
            vec![],
            0,
            vec![],
            0,
            zero_parent_header(),
        )
        .unwrap();
        assert_eq!(proof.hash_block, proof.parent_header.hash());
    }

    #[test]
    fn merkle_branch_order_matches_auxpow_check() {
        let leaf = sha256d(b"leaf");
        let sibling = sha256d(b"sibling");
        let left = apply_merkle_branch(leaf, &[sibling], 0);
        let right = apply_merkle_branch(leaf, &[sibling], 1);

        let mut left_bytes = [0_u8; 64];
        left_bytes[..32].copy_from_slice(leaf.as_wire_le_bytes());
        left_bytes[32..].copy_from_slice(sibling.as_wire_le_bytes());
        assert_eq!(left, sha256d(&left_bytes));

        let mut right_bytes = [0_u8; 64];
        right_bytes[..32].copy_from_slice(sibling.as_wire_le_bytes());
        right_bytes[32..].copy_from_slice(leaf.as_wire_le_bytes());
        assert_eq!(right, sha256d(&right_bytes));
    }

    #[test]
    fn esf_shape_rejects_wrong_branch_depth() {
        let proof = EsfAuxPowProof::new(
            SerializedParentCoinbaseTx::new(vec![1]).unwrap(),
            vec![],
            0,
            vec![],
            0,
            zero_parent_header(),
        )
        .unwrap();
        assert!(matches!(
            proof.validate_chain_shape(2, 0),
            Err(AuxPowError::ChainBranchSizeMismatch { .. })
        ));
    }

    #[test]
    fn parent_coinbase_must_be_index_zero() {
        assert!(matches!(
            EsfAuxPowProof::new(
                SerializedParentCoinbaseTx::new(vec![1]).unwrap(),
                vec![],
                1,
                vec![],
                0,
                zero_parent_header(),
            ),
            Err(AuxPowError::InvalidParentIndex(1))
        ));
    }
}
