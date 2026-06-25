use anyhow::Result;
use reth_stateless::StatelessInput;
use sha2::{Digest, Sha256};

use crate::guest::canonical_io;

/// Returns canonical input and expected output. The reth guest entrypoint writes
/// the sha256 digest of the canonical output bytes, since some zkVMs cap public
/// values at 32 bytes.
pub fn io(input: &StatelessInput, success: bool) -> Result<(Vec<u8>, Vec<u8>)> {
    let (input_bytes, output_bytes) = canonical_io(input, success)?;
    Ok((input_bytes, Sha256::digest(output_bytes).to_vec()))
}
