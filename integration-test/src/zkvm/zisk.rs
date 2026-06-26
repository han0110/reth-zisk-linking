//! ZisK in-process execution, driving the ziskemu library directly as a crate.

use anyhow::{Result, anyhow, bail};
use zisk_core::{Riscv2zisk, ZiskRom};
use ziskemu::{Emu, EmuOptions};

pub struct ZiskExecutor {
    rom: ZiskRom,
}

impl ZiskExecutor {
    pub fn new(elf: &[u8]) -> Result<Self> {
        let rom = Riscv2zisk::new(elf)
            .run()
            .map_err(|error| anyhow!("riscv2zisk: {error}"))?;
        Ok(Self { rom })
    }

    pub fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        let opts = EmuOptions::default();
        let mut emu = Emu::new(&self.rom);
        emu.ctx = emu.create_emu_context(framed_stdin(input), &opts);
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| emu.run_fast(&opts))).map_err(
            |err| {
                let msg = err
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| err.downcast_ref::<&str>().copied())
                    .unwrap_or("unknown");
                anyhow!("ziskemu panicked: {msg}")
            },
        )?;
        if !emu.ctx.inst_ctx.end {
            bail!("ziskemu did not terminate");
        }
        if emu.ctx.inst_ctx.error {
            bail!("ziskemu reported an error");
        }
        Ok(emu.get_output_8())
    }
}

/// Wraps the input in ZisK's stdin framing, an 8 byte little endian length
/// followed by the data padded to a multiple of 8.
fn framed_stdin(data: &[u8]) -> Vec<u8> {
    let len = (8 + data.len()).next_multiple_of(8);
    let mut buf = Vec::with_capacity(len);
    buf.extend_from_slice(&(data.len() as u64).to_le_bytes());
    buf.extend_from_slice(data);
    buf.resize(len, 0);
    buf
}
