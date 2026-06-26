use anyhow::Result;

use crate::fixture::Fixture;

/// Returns canonical input and the expected `new_payload_request_root` plus
/// `successful_validation` prefix. The zesu guest hardcodes its own chain config
/// in the SSZ output tail, so only the leading 33 bytes (root + success) are
/// compared.
pub fn io(fixture: &Fixture) -> Result<(Vec<u8>, Vec<u8>)> {
    Ok((fixture.input_bytes.clone(), fixture.output_bytes[..33].to_vec()))
}
