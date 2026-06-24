use anyhow::Result;
use sha2::{Digest, Sha256};
use stateless::StatelessInput;
use stateless_validator_reth::guest::{
    StatelessValidatorOutput, StatelessValidatorRethInput, new_payload_request::NativeSha256Hasher,
};

/// Returns input and expected output.
pub fn io(stateless_input: &StatelessInput, success: bool) -> Result<(Vec<u8>, Vec<u8>)> {
    let input = StatelessValidatorRethInput::new(stateless_input, success)?;
    let root = input
        .new_payload_request
        .tree_hash_root(&NativeSha256Hasher);
    let input = bincode::serde::encode_to_vec(&input, bincode::config::legacy())?;
    let expected_output = Sha256::digest(StatelessValidatorOutput::new(root, success).serialize());
    Ok((input, expected_output.to_vec()))
}
