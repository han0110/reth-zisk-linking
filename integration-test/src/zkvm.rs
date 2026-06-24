pub mod sp1;
pub mod zisk;

use anyhow::Result;
use clap::ValueEnum;

/// zkVM backend under test.
#[derive(Clone, Copy, ValueEnum)]
pub enum Zkvm {
    Sp1,
    Zisk,
}

impl Zkvm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sp1 => "sp1",
            Self::Zisk => "zisk",
        }
    }
}

/// A zkVM executor that runs a guest ELF in-process. Shareable across threads:
/// SP1 holds a pool of reusable executors, ZisK builds a fresh emulator per run.
pub enum Executor {
    Sp1(sp1::Sp1Pool),
    Zisk(zisk::ZiskExecutor),
}

impl Executor {
    /// Loads a guest ELF for the given backend.
    pub fn new(zkvm: Zkvm, elf: &[u8]) -> Result<Self> {
        Ok(match zkvm {
            Zkvm::Sp1 => Self::Sp1(sp1::Sp1Pool::new(elf, pool_size())?),
            Zkvm::Zisk => Self::Zisk(zisk::ZiskExecutor::new(elf)?),
        })
    }

    /// Runs the guest on the raw input and returns the committed public output.
    /// SP1 takes the input as one hint chunk; ZisK prepends an 8 byte length frame.
    pub fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        match self {
            Self::Sp1(pool) => pool.execute(input),
            Self::Zisk(executor) => executor.execute(input),
        }
    }
}

/// Concurrent executions, bounding the SP1 pool size.
fn pool_size() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get()).min(32)
}
