pub mod nethermind;
pub mod reth;
pub mod zesu;

use anyhow::Result;
use clap::ValueEnum;
use reth_stateless::StatelessInput as RethStatelessInput;
use stateless_validator_common::{
    HashTreeRoot, Sha2Hasher, SszEncode as _,
    guest::{StatelessInput, StatelessValidationResult},
};

/// Execution layer guest under test.
#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Guest {
    Reth,
    Zesu,
    Nethermind,
}

impl Guest {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reth => "reth",
            Self::Zesu => "zesu",
            Self::Nethermind => "nethermind",
        }
    }

    /// Builds the canonical guest input and the natively computed expected
    /// output for a fixture.
    pub fn io(self, input: &RethStatelessInput, success: bool) -> Result<(Vec<u8>, Vec<u8>)> {
        match self {
            Self::Reth => reth::io(input, success),
            Self::Zesu => zesu::io(input, success),
            Self::Nethermind => nethermind::io(input, success),
        }
    }
}

/// Converts a fixture into canonical schema-prefixed SSZ input bytes and the
/// expected canonical SSZ `StatelessValidationResult` output bytes. The success
/// bit is taken from the fixture oracle, while the payload root and echoed chain
/// config come from the canonical input, mirroring what the guest emits.
pub(crate) fn canonical_io(
    input: &RethStatelessInput,
    success: bool,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let canonical = StatelessInput::try_from_reth(input)?;
    let input_bytes = canonical.to_schema_prefixed_ssz();
    let root = canonical.new_payload_request.hash_tree_root(&Sha2Hasher);
    let result = StatelessValidationResult::new(root, success, canonical.chain_config);
    let output_bytes = result.to_ssz();
    Ok((input_bytes, output_bytes))
}
