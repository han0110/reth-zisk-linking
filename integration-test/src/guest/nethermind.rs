use anyhow::{Result, bail};
use stateless_validator_common::guest::{
    StatelessInput, input::new_payload_request::NewPayloadRequest,
};

use crate::fixture::Fixture;

/// Returns Nethermind-format input and the full expected output.
///
/// Nethermind's r7 guest selects the payload type from the 2-byte schema id
/// (0 = pre-Amsterdam V3, 1 = Amsterdam V4) and decodes a typed 4-field
/// `NewPayloadRequest`, whereas ere-guests always prefixes `0x0001`. Every SSZ
/// container is byte-identical between the two, and the ere-guests `ElectraFulu`
/// (V3) and `Gloas` (V4) variants exactly match Nethermind's 4-field wire shape,
/// so only the 2-byte schema prefix is rewritten. The variant is recovered by
/// re-parsing the canonical input bytes. The pre-Electra shapes have no
/// Nethermind r7 equivalent. Unlike zesu, Nethermind echoes the input chain
/// config in its output (it derives the active fork from the input), so the full
/// `StatelessValidationResult` matches and is compared in its entirety.
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
