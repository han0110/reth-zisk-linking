//! Fixture driven end to end tests for execution layer guests linked against
//! the ZisK runtime.
//!
//! Each fixture under the input directory holds a reth StatelessInput
//! together with the expected validation verdict. The fixture is converted
//! into the guest input format selected by the --el flag, wrapped in the
//! ZisK length prefixed stdin framing, and executed on ziskemu against the
//! linked guest ELF. The emulated public output is compared against the
//! expected bytes computed natively.

mod reth;
mod stateless_ssz;
mod zesu;

use std::{
    fs,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Result, bail};
use clap::{Parser, ValueEnum};
use rayon::prelude::*;
use serde::Deserialize;
use stateless::StatelessInput;

/// Execution layer guest under test.
#[derive(Clone, Copy, ValueEnum)]
enum El {
    Reth,
    Zesu,
}

impl El {
    fn elf_name(self) -> &'static str {
        match self {
            Self::Reth => "reth-zisk.elf",
            Self::Zesu => "zesu-zisk.elf",
        }
    }
}

#[derive(Parser)]
struct Args {
    /// Execution layer guest under test.
    #[arg(long, value_enum)]
    el: El,
    /// Guest ELF to emulate, defaults to the linked ELF of the selected
    /// guest at the repository root.
    #[arg(long)]
    elf_path: Option<PathBuf>,
    /// Directory of fixture json files, defaults to fixtures at the
    /// repository root.
    #[arg(long)]
    input_dir: Option<PathBuf>,
    /// ziskemu binary, defaults to the release build in the zisk submodule.
    #[arg(long)]
    ziskemu: Option<PathBuf>,
    /// Substring filter on fixture file names.
    #[arg(long)]
    filter: Option<String>,
}

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relative)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let elf_path = args
        .elf_path
        .unwrap_or_else(|| repo_path(args.el.elf_name()));
    let input_dir = args.input_dir.unwrap_or_else(|| repo_path("fixtures"));
    let ziskemu = args
        .ziskemu
        .unwrap_or_else(|| repo_path("zisk/target/release/ziskemu"))
        .canonicalize()?;

    let mut paths = fs::read_dir(&input_dir)?
        .filter_map(|result| result.ok().map(|file| file.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .filter(|path| match &args.filter {
            Some(needle) => path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|name| name.contains(needle)),
            None => true,
        })
        .collect::<Vec<_>>();
    paths.sort();
    if paths.is_empty() {
        bail!("no *.json fixtures in {}", input_dir.display());
    }

    let total = paths.len();
    let done = AtomicUsize::new(0);
    let failures = paths
        .par_iter()
        .filter_map(|path| {
            let result = emulate(args.el, &elf_path, &ziskemu, path);
            let done = done.fetch_add(1, Ordering::Relaxed) + 1;
            match result {
                Ok(name) => {
                    println!("[{done}/{total}] {name}");
                    None
                }
                Err(error) => {
                    println!("[{done}/{total}] FAILED {}\n{error}", path.display());
                    Some(path)
                }
            }
        })
        .collect::<Vec<_>>();

    if !failures.is_empty() {
        bail!("{} of {total} fixtures failed", failures.len());
    }
    println!("all {total} fixtures passed");
    Ok(())
}

#[derive(Deserialize)]
struct StatelessValidatorFixture {
    name: String,
    stateless_input: StatelessInput,
    success: bool,
}

fn emulate(el: El, elf_path: &Path, ziskemu: &Path, path: &Path) -> Result<String> {
    let tmpdir = tempfile::tempdir()?;
    let input_path = tmpdir.path().join("input");
    let output_path = tmpdir.path().join("output");

    let (name, expected_output) = {
        let StatelessValidatorFixture {
            name,
            stateless_input,
            success,
        } = serde_json::from_slice(&fs::read(path)?)?;
        let (input, expected_output) = match el {
            El::Reth => reth::io(&stateless_input, success)?,
            El::Zesu => zesu::io(&stateless_input, success)?,
        };
        fs::write(&input_path, frame_input(&input))?;
        (name, expected_output)
    };

    let run = Command::new(ziskemu)
        .arg("-e")
        .arg(elf_path)
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()?;

    if !run.status.success() {
        bail!(
            "ziskemu failed for {name} (exit {:?}, signal {:?})\n{}{}",
            run.status.code(),
            run.status.signal(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr),
        );
    }

    let actual = fs::read(&output_path)?;
    if actual.get(..expected_output.len()) != Some(expected_output.as_slice()) {
        bail!(
            "output mismatch for {name}\nexpected {}\ngot      {}\nemulator output\n{}",
            hex::encode(&expected_output),
            hex::encode(actual.get(..expected_output.len()).unwrap_or(&actual)),
            String::from_utf8_lossy(&run.stdout),
        );
    }

    Ok(name)
}

/// Wraps the guest input in the ZisK stdin framing, which is an 8 byte
/// little endian length followed by the input padded to a multiple of 8.
fn frame_input(input: &[u8]) -> Vec<u8> {
    let len = (8 + input.len()).next_multiple_of(8);
    let mut buf = Vec::with_capacity(len);
    buf.extend_from_slice(&(input.len() as u64).to_le_bytes());
    buf.extend_from_slice(input);
    buf.resize(len, 0);
    buf
}
