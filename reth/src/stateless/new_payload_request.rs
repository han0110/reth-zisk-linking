use alloc::{vec, vec::Vec};

use libssz_derive::{HashTreeRoot, SszDecode, SszEncode};
use libssz_merkle::{HashTreeRoot, Sha256Hasher};
use libssz_types::SszList;
use revm::precompile::crypto;
use serde::{Deserialize, Serialize};

pub type Hash32 = [u8; 32];
pub type Bytes48 = [u8; 48];
pub type Bytes96 = [u8; 96];
pub type Address20 = [u8; 20];
pub type Uint256Bytes = [u8; 32];
pub type LogsBloom = [u8; 256];
pub type ExtraData = SszList<u8, 32>;

pub const MAX_WITHDRAWALS_PER_PAYLOAD: usize = 16;
pub const MAX_TRANSACTIONS_PER_PAYLOAD: usize = 1024 * 1024;
pub const MAX_BYTES_PER_TRANSACTION: usize = MAX_TRANSACTIONS_PER_PAYLOAD * 1024;
pub const MAX_BLOB_COMMITMENTS_PER_BLOCK: usize = 4096;
pub const MAX_DEPOSIT_REQUESTS_PER_PAYLOAD: usize = 8192;
pub const MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: usize = 16;
pub const MAX_CONSOLIDATION_REQUESTS_PER_PAYLOAD: usize = 2;

pub type BlockAccessList = SszList<u8, MAX_BYTES_PER_TRANSACTION>;
pub type Transaction = SszList<u8, MAX_BYTES_PER_TRANSACTION>;
pub type Transactions = SszList<Transaction, MAX_TRANSACTIONS_PER_PAYLOAD>;
pub type Withdrawals = SszList<Withdrawal, MAX_WITHDRAWALS_PER_PAYLOAD>;
pub type VersionedHashes = SszList<Hash32, MAX_BLOB_COMMITMENTS_PER_BLOCK>;
pub type DepositRequests = SszList<DepositRequest, MAX_DEPOSIT_REQUESTS_PER_PAYLOAD>;
pub type WithdrawalRequests = SszList<WithdrawalRequest, MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD>;
pub type ConsolidationRequests =
    SszList<ConsolidationRequest, MAX_CONSOLIDATION_REQUESTS_PER_PAYLOAD>;

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct Withdrawal {
    pub index: u64,
    pub validator_index: u64,
    pub address: Address20,
    pub amount: u64,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct DepositRequest {
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub pubkey: Bytes48,
    pub withdrawal_credentials: Hash32,
    pub amount: u64,
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub signature: Bytes96,
    pub index: u64,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct WithdrawalRequest {
    pub source_address: Address20,
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub validator_pubkey: Bytes48,
    pub amount: u64,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct ConsolidationRequest {
    pub source_address: Address20,
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub source_pubkey: Bytes48,
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub target_pubkey: Bytes48,
}

#[derive(Debug, Clone, Default, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct ExecutionRequests {
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub deposits: DepositRequests,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub withdrawals: WithdrawalRequests,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub consolidations: ConsolidationRequests,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct ExecutionPayloadV1 {
    pub parent_hash: Hash32,
    pub fee_recipient: Address20,
    pub state_root: Hash32,
    pub receipts_root: Hash32,
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub logs_bloom: LogsBloom,
    pub prev_randao: Hash32,
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub extra_data: ExtraData,
    pub base_fee_per_gas: Uint256Bytes,
    pub block_hash: Hash32,
    #[serde(with = "crate::stateless::serde_wrappers::nested_ssz_list")]
    pub transactions: Transactions,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct ExecutionPayloadV2 {
    pub parent_hash: Hash32,
    pub fee_recipient: Address20,
    pub state_root: Hash32,
    pub receipts_root: Hash32,
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub logs_bloom: LogsBloom,
    pub prev_randao: Hash32,
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub extra_data: ExtraData,
    pub base_fee_per_gas: Uint256Bytes,
    pub block_hash: Hash32,
    #[serde(with = "crate::stateless::serde_wrappers::nested_ssz_list")]
    pub transactions: Transactions,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub withdrawals: Withdrawals,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct ExecutionPayloadV3 {
    pub parent_hash: Hash32,
    pub fee_recipient: Address20,
    pub state_root: Hash32,
    pub receipts_root: Hash32,
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub logs_bloom: LogsBloom,
    pub prev_randao: Hash32,
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub extra_data: ExtraData,
    pub base_fee_per_gas: Uint256Bytes,
    pub block_hash: Hash32,
    #[serde(with = "crate::stateless::serde_wrappers::nested_ssz_list")]
    pub transactions: Transactions,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub withdrawals: Withdrawals,
    pub blob_gas_used: u64,
    pub excess_blob_gas: u64,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct ExecutionPayloadV4 {
    pub parent_hash: Hash32,
    pub fee_recipient: Address20,
    pub state_root: Hash32,
    pub receipts_root: Hash32,
    #[serde(with = "crate::stateless::serde_wrappers::bytes_array")]
    pub logs_bloom: LogsBloom,
    pub prev_randao: Hash32,
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub extra_data: ExtraData,
    pub base_fee_per_gas: Uint256Bytes,
    pub block_hash: Hash32,
    #[serde(with = "crate::stateless::serde_wrappers::nested_ssz_list")]
    pub transactions: Transactions,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub withdrawals: Withdrawals,
    pub blob_gas_used: u64,
    pub excess_blob_gas: u64,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub block_access_list: BlockAccessList,
    pub slot_number: u64,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct NewPayloadRequestBellatrix {
    pub execution_payload: ExecutionPayloadV1,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct NewPayloadRequestCapella {
    pub execution_payload: ExecutionPayloadV2,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct NewPayloadRequestDeneb {
    pub execution_payload: ExecutionPayloadV3,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub versioned_hashes: VersionedHashes,
    pub parent_beacon_block_root: Hash32,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct NewPayloadRequestElectraFulu {
    pub execution_payload: ExecutionPayloadV3,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub versioned_hashes: VersionedHashes,
    pub parent_beacon_block_root: Hash32,
    pub execution_requests: ExecutionRequests,
}

#[derive(Debug, Clone, HashTreeRoot, SszEncode, SszDecode, Serialize, Deserialize)]
pub struct NewPayloadRequestAmsterdam {
    pub execution_payload: ExecutionPayloadV4,
    #[serde(with = "crate::stateless::serde_wrappers::ssz_list")]
    pub versioned_hashes: VersionedHashes,
    pub parent_beacon_block_root: Hash32,
    pub execution_requests: ExecutionRequests,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewPayloadRequest {
    Bellatrix(NewPayloadRequestBellatrix),
    Capella(NewPayloadRequestCapella),
    Deneb(NewPayloadRequestDeneb),
    ElectraFulu(NewPayloadRequestElectraFulu),
    Amsterdam(NewPayloadRequestAmsterdam),
}

impl NewPayloadRequest {
    pub fn tree_hash_root(&self, hasher: &impl Sha256Hasher) -> [u8; 32] {
        match self {
            NewPayloadRequest::Bellatrix(req) => req.hash_tree_root(hasher),
            NewPayloadRequest::Capella(req) => req.hash_tree_root(hasher),
            NewPayloadRequest::Deneb(req) => req.hash_tree_root(hasher),
            NewPayloadRequest::ElectraFulu(req) => req.hash_tree_root(hasher),
            NewPayloadRequest::Amsterdam(req) => req.hash_tree_root(hasher),
        }
    }
}

pub fn compute_requests_hash(requests: &ExecutionRequests) -> [u8; 32] {
    use libssz::SszEncode;
    let mut outer_bytes = Vec::new();

    let mut deposits_bytes = vec![0x00u8];
    for deposit in requests.deposits.iter() {
        deposits_bytes.extend(deposit.to_ssz());
    }
    if deposits_bytes.len() > 1 {
        outer_bytes.extend_from_slice(&crypto().sha256(&deposits_bytes));
    }

    let mut withdrawals_bytes = vec![0x01u8];
    for withdrawal in requests.withdrawals.iter() {
        withdrawals_bytes.extend(withdrawal.to_ssz());
    }
    if withdrawals_bytes.len() > 1 {
        outer_bytes.extend_from_slice(&crypto().sha256(&withdrawals_bytes));
    }

    let mut consolidations_bytes = vec![0x02u8];
    for consolidation in requests.consolidations.iter() {
        consolidations_bytes.extend(consolidation.to_ssz());
    }
    if consolidations_bytes.len() > 1 {
        outer_bytes.extend_from_slice(&crypto().sha256(&consolidations_bytes));
    }

    crypto().sha256(&outer_bytes)
}
