pub mod nethermind;
pub mod reth;
pub mod zesu;

use anyhow::Result;
use clap::ValueEnum;

use crate::fixture::Fixture;

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

    /// Builds the guest input and natively computed expected output, transforming
    /// the fixture's canonical bytes for the guest's wire format and output convention.
    pub fn io(self, fixture: &Fixture) -> Result<(Vec<u8>, Vec<u8>)> {
        match self {
            Self::Reth => reth::io(fixture),
            Self::Zesu => zesu::io(fixture),
            Self::Nethermind => nethermind::io(fixture),
        }
    }
}
