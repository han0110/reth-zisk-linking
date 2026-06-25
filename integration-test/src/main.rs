//! Fixture-driven tests for EL guests across zkVM backends.
//!
//! Each fixture is converted to the selected guest's input, executed on the
//! selected backend, and the committed output is compared against the expected
//! bytes. Choose the backend with --zkvm and the guest with --el.

mod guest;
mod zkvm;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use rayon::prelude::*;
use reth_stateless::StatelessInput;
use serde::Deserialize;

use crate::{
    guest::Guest,
    zkvm::{Executor, Zkvm},
};

#[derive(Parser)]
struct Args {
    /// zkVM backend under test.
    #[arg(long, value_enum)]
    zkvm: Zkvm,
    /// Execution layer guest under test.
    #[arg(long, value_enum)]
    el: Guest,
    /// Guest ELF, defaults to build/<el>-<zkvm>.elf.
    #[arg(long)]
    elf_path: Option<PathBuf>,
    /// Directory of fixture json files, defaults to fixtures.
    #[arg(long)]
    input_dir: Option<PathBuf>,
    /// Substring filter on fixture file names.
    #[arg(long)]
    filter: Option<String>,
    /// Cap the number of fixtures executed.
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct StatelessValidatorFixture {
    name: String,
    stateless_input: StatelessInput,
    success: bool,
}

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(relative)
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.el == Guest::Nethermind && !matches!(args.zkvm, Zkvm::Zisk) {
        bail!("--el nethermind is a ZisK guest ELF and is only supported with --zkvm zisk");
    }
    let elf_path = args.elf_path.clone().unwrap_or_else(|| {
        repo_path(&format!("build/{}-{}.elf", args.el.as_str(), args.zkvm.as_str()))
    });
    let input_dir = args.input_dir.clone().unwrap_or_else(|| repo_path("fixtures"));

    let elf = fs::read(&elf_path).with_context(|| format!("read elf {}", elf_path.display()))?;
    println!(
        "zkvm={} el={} elf={} ({} bytes)",
        args.zkvm.as_str(),
        args.el.as_str(),
        elf_path.display(),
        elf.len()
    );

    let mut paths = fs::read_dir(&input_dir)?
        .filter_map(|result| result.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .filter(|path| match &args.filter {
            Some(needle) => path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|name| name.contains(needle)),
            None => true,
        })
        .collect::<Vec<_>>();
    paths.sort();
    if let Some(limit) = args.limit {
        paths.truncate(limit);
    }
    if paths.is_empty() {
        bail!("no *.json fixtures in {}", input_dir.display());
    }

    let executor = Executor::new(args.zkvm, &elf)?;
    let total = paths.len();
    let results: Vec<Result<String, String>> = paths
        .par_iter()
        .map(|path| run_fixture(path, args.el, &executor))
        .collect();

    let mut failures = 0;
    for (index, result) in results.iter().enumerate() {
        match result {
            Ok(name) => println!("[{}/{total}] {name}", index + 1),
            Err(message) => {
                println!("[{}/{total}] FAILED {message}", index + 1);
                failures += 1;
            }
        }
    }

    if failures != 0 {
        bail!("{failures} of {total} fixtures failed");
    }
    println!("all {total} fixtures passed");
    Ok(())
}

/// Runs one fixture, returning its name on success or a failure message.
fn run_fixture(path: &Path, el: Guest, executor: &Executor) -> Result<String, String> {
    let fixture: StatelessValidatorFixture = fs::read(path)
        .map_err(|err| err.to_string())
        .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|err| err.to_string()))
        .map_err(|err| format!("{}: {err}", path.display()))?;
    let name = fixture.name;
    let (input, expected) = el
        .io(&fixture.stateless_input, fixture.success)
        .map_err(|err| format!("{name} (io)\n{err:?}"))?;
    let output = executor
        .execute(&input)
        .map_err(|err| format!("{name}\n{err:?}"))?;
    if output.get(..expected.len()) == Some(expected.as_slice()) {
        Ok(name)
    } else {
        Err(format!(
            "{name}\n  expected {}\n  got      {}",
            hex::encode(&expected),
            hex::encode(output.get(..expected.len()).unwrap_or(&output)),
        ))
    }
}
