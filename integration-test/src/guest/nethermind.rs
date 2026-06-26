use anyhow::{Result, bail};
use stateless_validator_common::guest::{
    StatelessInput, input::new_payload_request::NewPayloadRequest,
};

use crate::fixture::Fixture;

/// Returns Nethermind-format input and the full expected output.
///
/// Nethermind's r7 guest selects the payload type from a 2-byte schema id
/// (0 = `ElectraFulu` V3, 1 = `Gloas` V4) where ere-guests always prefix
/// `0x0001`. The SSZ containers are byte-identical, so only the prefix is
/// rewritten after recovering the variant. Nethermind echoes the input chain
/// config in its output, so the full `StatelessValidationResult` is compared.
pub fn io(fixture: &Fixture) -> Result<(Vec<u8>, Vec<u8>)> {
    let canonical = StatelessInput::from_schema_prefixed_ssz(&fixture.input_bytes)?;

    let schema_id: [u8; 2] = match &canonical.new_payload_request {
        NewPayloadRequest::ElectraFulu(_) => [0x00, 0x00],
        NewPayloadRequest::Gloas(_) => [0x00, 0x01],
        _ => bail!("nethermind r7 supports only ElectraFulu (V3) and Gloas (V4) payloads"),
    };

    let mut input_bytes = fixture.input_bytes.clone();
    input_bytes[0..2].copy_from_slice(&schema_id);

    Ok((input_bytes, fixture.output_bytes.clone()))
}
