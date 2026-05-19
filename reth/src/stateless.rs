use alloy_genesis::Genesis;
use alloy_primitives::sync::Arc;
use reth_chainspec::ChainSpec;
use reth_evm_ethereum::EthEvmConfig;
use revm::precompile::crypto;
use stateless::stateless_validation_with_trie;
use tries::zeth::SparseState;

use crate::{
    crypto::ZkVMCrypto,
    stateless::{
        input::StatelessValidatorRethInput, output::StatelessValidatorOutput,
        payload_to_block::new_payload_request_to_block,
    },
};

mod input;
mod new_payload_request;
mod output;
mod payload_to_block;
mod serde_wrappers;

pub fn compute(input: &[u8]) -> [u8; 32] {
    let (input, _) = bincode::serde::decode_from_slice(input, bincode::config::legacy()).unwrap();
    let output = compute_inner(input);
    crypto().sha256(&output.serialize())
}

pub fn compute_inner(input: StatelessValidatorRethInput) -> StatelessValidatorOutput {
    let new_payload_request_root = input.new_payload_request.tree_hash_root(&ZkVMCrypto);

    let genesis = Genesis {
        config: input.chain_config.clone(),
        ..Default::default()
    };
    let chain_spec: Arc<ChainSpec> = Arc::new(genesis.into());
    let evm_config = EthEvmConfig::new(chain_spec.clone());

    let sealed_block =
        match new_payload_request_to_block(input.new_payload_request, chain_spec.clone()) {
            Ok(sb) => sb,
            Err(_) => return StatelessValidatorOutput::new(new_payload_request_root, false),
        };
    let block = sealed_block.into_block();

    match stateless_validation_with_trie::<SparseState, _, _>(
        block,
        input.public_keys,
        input.witness,
        chain_spec,
        evm_config,
    ) {
        Ok(_) => StatelessValidatorOutput::new(new_payload_request_root, true),
        Err(_) => StatelessValidatorOutput::new(new_payload_request_root, false),
    }
}
