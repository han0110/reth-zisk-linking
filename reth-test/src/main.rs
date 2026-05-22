use std::{
    fs,
    os::unix::process::ExitStatusExt,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Result, bail};
use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stateless::StatelessInput;
use stateless_validator_reth::guest::{
    StatelessValidatorOutput, StatelessValidatorRethInput, new_payload_request::NativeSha256Hasher,
};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    elf_path: PathBuf,
    #[arg(long)]
    input_dir: PathBuf,
    #[arg(long)]
    filter: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut paths = fs::read_dir(&args.input_dir)?
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
    let tmpdir = tempfile::tempdir()?;
    let input_path = tmpdir.path().join("input");
    let output_path = tmpdir.path().join("output");

    let (name, expected) = {
        let StatelessValidatorFixture {
            name,
            stateless_input,
            success,
        } = serde_json::from_slice(&fs::read(path)?)?;
        let input = StatelessValidatorRethInput::new(&stateless_input, success)?;
        let root = input
            .new_payload_request
            .tree_hash_root(&NativeSha256Hasher);
        fs::write(&input_path, serialize_input(&input)?)?;
        let expected = Sha256::digest(StatelessValidatorOutput::new(root, success).serialize());
        (name, expected)
    };

    let status = Command::new("ziskemu")
        .arg("-e")
        .arg(elf_path)
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .status()?;

    if !status.success() {
        bail!(
            "ziskemu failed for {name} (exit {:?}, signal {:?})",
            status.code(),
            status.signal(),
        );
    }

    let actual = fs::read(&output_path)?;
    if actual.get(..32) != Some(&expected) {
        bail!(
            "output mismatch for {name}: expected {}, got {}",
            hex::encode(expected),
            hex::encode(actual.get(..32).unwrap_or(&actual))
        );
    }

    Ok(name)
}

fn serialize_input(stdin: impl Serialize) -> Result<Vec<u8>> {
    let stdin = bincode::serde::encode_to_vec(&stdin, bincode::config::legacy())?;
    let len = (8 + stdin.len()).next_multiple_of(8);
    let mut buf = Vec::with_capacity(len);
    buf.extend_from_slice(&(stdin.len() as u64).to_le_bytes());
    buf.extend_from_slice(&stdin);
    buf.resize(len, 0);
    Ok(buf)
}
