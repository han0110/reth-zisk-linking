pub mod reth;
pub mod zesu;

mod stateless_ssz;

use anyhow::Result;
use clap::ValueEnum;
use stateless::StatelessInput;

/// Execution layer guest under test.
#[derive(Clone, Copy, ValueEnum)]
pub enum Guest {
    Reth,
    Zesu,
}

impl Guest {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reth => "reth",
            Self::Zesu => "zesu",
        }
    }

    /// Builds the guest input and the natively computed expected output for a
    /// fixture.
    pub fn io(self, input: &StatelessInput, success: bool) -> Result<(Vec<u8>, Vec<u8>)> {
        match self {
            Self::Reth => reth::io(input, success),
            Self::Zesu => zesu::io(input, success),
        }
    }
}
