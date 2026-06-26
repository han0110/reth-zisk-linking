use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::fixture::Fixture;

/// Returns canonical input and expected output. The reth guest entrypoint writes
/// the sha256 digest of the canonical output bytes, since some zkVMs cap public
/// values at 32 bytes.
pub fn io(fixture: &Fixture) -> Result<(Vec<u8>, Vec<u8>)> {
    Ok((
        fixture.input_bytes.clone(),
        Sha256::digest(&fixture.output_bytes).to_vec(),
    ))
}
