//! Expected StatelessValidationResult bytes for one fixture.

use libssz::SszEncode as _;
use libssz_merkle::Sha2Hasher;
use libssz_types::SszList;

use crate::guest::stateless_ssz::{
    BlobSchedule, ChainConfig, ForkActivation, ForkConfig, StatelessInput,
    StatelessValidationResult,
};

/// Builds the expected guest output for the given success flag.
pub fn expected_output(input: &StatelessInput, success: bool) -> Vec<u8> {
    StatelessValidationResult {
        new_payload_request_root: input.new_payload_request.tree_hash_root(&Sha2Hasher),
        successful_validation: success,
        chain_config: mainnet_amsterdam_chain_config(),
    }
    .to_ssz()
}

/// The zesu guest serializes this constant chain config regardless of the
/// input, see SSZ_CHAIN_CONFIG_AMSTERDAM_MAINNET in zesu.
fn mainnet_amsterdam_chain_config() -> ChainConfig {
    ChainConfig {
        chain_id: 1,
        active_fork: ForkConfig {
            fork: 24,
            activation: ForkActivation {
                block_number: SszList::new(),
                timestamp: vec![0].try_into().unwrap(),
            },
            blob_schedule: vec![BlobSchedule {
                target: 14,
                max: 21,
                base_fee_update_fraction: 11_684_671,
            }]
            .try_into()
            .unwrap(),
        },
    }
}
