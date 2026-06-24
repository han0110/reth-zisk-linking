//! Implementation of https://github.com/ethereum/execution-specs/blob/projects/zkevm/src/ethereum/forks/amsterdam/stateless_ssz.py.

use libssz::SszEncode as _;
use libssz_derive::SszEncode;
use libssz_merkle::{Node, Sha256Hasher};
use libssz_types::SszList;

pub const MAX_PUBLIC_KEYS: usize = 1 << 20;
pub const MAX_WITNESS_NODES: usize = 1 << 20;
pub const MAX_WITNESS_CODES: usize = 1 << 16;
pub const MAX_WITNESS_HEADERS: usize = 256;
pub const MAX_BYTES_PER_WITNESS_NODE: usize = 1 << 20;
pub const MAX_BYTES_PER_CODE: usize = 1 << 24;
pub const MAX_BYTES_PER_HEADER: usize = 1 << 10;
pub const MAX_OPTIONAL_FORK_ACTIVATION_VALUES: usize = 1;
pub const MAX_BLOB_SCHEDULES_PER_FORK: usize = 1;
pub const PUBLIC_KEY_BYTES: usize = 65;
pub const STATELESS_INPUT_SCHEMA_ID_BYTES: [u8; 2] = [0x00, 0x01];

#[derive(SszEncode)]
pub struct StatelessInput {
    pub new_payload_request: NewPayloadRequest,
    pub witness: ExecutionWitness,
    pub chain_config: ChainConfig,
    pub public_keys: SszList<[u8; PUBLIC_KEY_BYTES], MAX_PUBLIC_KEYS>,
}

impl StatelessInput {
    /// Serializes the schema id followed by the SSZ encoded container.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = STATELESS_INPUT_SCHEMA_ID_BYTES.to_vec();
        self.ssz_append(&mut out);
        out
    }
}

#[derive(SszEncode)]
pub struct StatelessValidationResult {
    pub new_payload_request_root: [u8; 32],
    pub successful_validation: bool,
    pub chain_config: ChainConfig,
}

/// SSZ wrapper.
pub struct NewPayloadRequest(
    pub stateless_validator_common::new_payload_request::NewPayloadRequest,
);

impl NewPayloadRequest {
    pub fn tree_hash_root(&self, hasher: &impl Sha256Hasher) -> Node {
        self.0.tree_hash_root(hasher)
    }
}

impl libssz::SszEncode for NewPayloadRequest {
    fn is_fixed_size() -> bool {
        false
    }

    fn fixed_size() -> usize {
        0
    }

    fn encoded_len(&self) -> usize {
        use stateless_validator_common::new_payload_request::NewPayloadRequest::*;
        match &self.0 {
            Bellatrix(request) => request.encoded_len(),
            Capella(request) => request.encoded_len(),
            Deneb(request) => request.encoded_len(),
            ElectraFulu(request) => request.encoded_len(),
            Amsterdam(request) => request.encoded_len(),
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        use stateless_validator_common::new_payload_request::NewPayloadRequest::*;
        match &self.0 {
            Bellatrix(request) => request.ssz_append(buf),
            Capella(request) => request.ssz_append(buf),
            Deneb(request) => request.ssz_append(buf),
            ElectraFulu(request) => request.ssz_append(buf),
            Amsterdam(request) => request.ssz_append(buf),
        }
    }
}

#[derive(SszEncode)]
pub struct ExecutionWitness {
    pub state: SszList<SszList<u8, MAX_BYTES_PER_WITNESS_NODE>, MAX_WITNESS_NODES>,
    pub codes: SszList<SszList<u8, MAX_BYTES_PER_CODE>, MAX_WITNESS_CODES>,
    pub headers: SszList<SszList<u8, MAX_BYTES_PER_HEADER>, MAX_WITNESS_HEADERS>,
}

#[derive(Clone, SszEncode)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub active_fork: ForkConfig,
}

#[derive(Clone, SszEncode)]
pub struct ForkConfig {
    pub fork: u64,
    pub activation: ForkActivation,
    pub blob_schedule: SszList<BlobSchedule, MAX_BLOB_SCHEDULES_PER_FORK>,
}

#[derive(Clone, SszEncode)]
pub struct ForkActivation {
    pub block_number: SszList<u64, MAX_OPTIONAL_FORK_ACTIVATION_VALUES>,
    pub timestamp: SszList<u64, MAX_OPTIONAL_FORK_ACTIVATION_VALUES>,
}

#[derive(Clone, SszEncode)]
pub struct BlobSchedule {
    pub target: u64,
    pub max: u64,
    pub base_fee_update_fraction: u64,
}
