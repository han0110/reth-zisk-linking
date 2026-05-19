use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{bail, Result};
use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use stateless::StatelessInput;
use stateless_validator_reth::guest::StatelessValidatorRethInput;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    elf_path: PathBuf,
    #[arg(long)]
    input_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut paths = fs::read_dir(&args.input_dir)
        .unwrap()
        .filter_map(|result| result.ok().map(|file| file.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    if paths.is_empty() {
        bail!("no *.json fixtures in {}", args.input_dir.display());
    }

    let total = paths.len();
    let done = AtomicUsize::new(0);
    paths.par_iter().try_for_each(|path| -> Result<()> {
        let name = emulate(&args.elf_path, path)?;
        let done = done.fetch_add(1, Ordering::Relaxed) + 1;
        println!("[{done}/{total}] {name}");
        Ok(())
    })?;

    Ok(())
}

#[derive(Deserialize)]
struct StatelessValidatorFixture {
    name: String,
    stateless_input: StatelessInput,
    success: bool,
}

fn emulate(elf_path: &PathBuf, path: &PathBuf) -> Result<String> {
    let StatelessValidatorFixture {
        name,
        stateless_input,
        success,
    } = serde_json::from_slice(&fs::read(path)?).unwrap();

    let input = StatelessValidatorRethInput::new(&stateless_input, success).unwrap();
    let mut input_file = tempfile::NamedTempFile::new()?;
    input_file.write_all(&serialize_input(&input))?;
    let input_path = input_file.into_temp_path();

    let status = Command::new("ziskemu")
        .arg("-e")
        .arg(elf_path)
        .arg("-i")
        .arg(&input_path)
        .status()
        .unwrap();

    if !status.success() {
        bail!("ziskemu failed for {name} (exit {:?})", status.code());
    }

    Ok(name)
}

fn serialize_input(stdin: impl Serialize) -> Vec<u8> {
    let stdin = bincode::serde::encode_to_vec(&stdin, bincode::config::legacy()).unwrap();
    let len = (8 + stdin.len()).next_multiple_of(8);
    let mut buf = Vec::with_capacity(len);
    buf.extend_from_slice(&(stdin.len() as u64).to_le_bytes());
    buf.extend_from_slice(&stdin);
    buf.resize(len, 0);
    buf
}
