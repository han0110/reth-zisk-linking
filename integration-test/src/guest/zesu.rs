use anyhow::Result;
use reth_stateless::StatelessInput;

use crate::guest::canonical_io;

/// Returns canonical input and the expected `new_payload_request_root` plus
/// `successful_validation` prefix. The zesu guest hardcodes its own chain config
/// in the SSZ output tail, so only the leading 33 bytes (root + success) are
/// compared.
pub fn io(input: &StatelessInput, success: bool) -> Result<(Vec<u8>, Vec<u8>)> {
    let (input_bytes, output_bytes) = canonical_io(input, success)?;
    Ok((input_bytes, output_bytes[..33].to_vec()))
}
