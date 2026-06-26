//! Fixture-driven tests for EL guests across zkVM backends.
//!
//! Each fixture is normalized to canonical SSZ input/output bytes, converted to
//! the selected guest's wire format, executed on the selected backend, and the
//! committed output is compared against the expected bytes. Choose the backend
//! with --zkvm and the guest with --el.

mod fixture;
mod guest;
mod zkvm;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{
    fixture::Fixture,
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
    /// Directory of fixture json files (searched recursively), defaults to fixtures.
    #[arg(long)]
    input_dir: Option<PathBuf>,
    /// Substring filter on fixture file paths.
    #[arg(long)]
    filter: Option<String>,
    /// Cap the number of fixtures executed.
    #[arg(long)]
    limit: Option<usize>,
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

    // Discover fixture files recursively so nested EEST trees are picked up.
    let mut files = WalkDir::new(&input_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".json") || name.ends_with(".json.zst"))
        })
        .filter(|path| match &args.filter {
            Some(needle) => path.to_string_lossy().contains(needle.as_str()),
            None => true,
        })
        .collect::<Vec<_>>();
    files.sort();
    if files.is_empty() {
        bail!("no *.json fixtures in {}", input_dir.display());
    }

    // Load and normalize every fixture, reporting unparseable files rather than
    // silently dropping them. Sorted by name for deterministic numbering.
    let loaded: Vec<Result<Vec<Fixture>>> =
        files.par_iter().map(|path| fixture::load(path)).collect();
    let mut fixtures = Vec::new();
    let mut load_errors = 0usize;
    for result in loaded {
        match result {
            Ok(mut batch) => fixtures.append(&mut batch),
            Err(err) => {
                if load_errors < 10 {
                    eprintln!("skip unparseable fixture: {err:?}");
                }
                load_errors += 1;
            }
        }
    }
    if load_errors != 0 {
        eprintln!("skipped {load_errors} unparseable fixture file(s)");
    }
    fixtures.sort_by(|a, b| a.name.cmp(&b.name));
    if let Some(limit) = args.limit {
        fixtures.truncate(limit);
    }
    if fixtures.is_empty() {
        bail!("no fixtures parsed from {}", input_dir.display());
    }

    let executor = Executor::new(args.zkvm, &elf)?;
    let total = fixtures.len();
    let results: Vec<Result<String, String>> = fixtures
        .par_iter()
        .map(|fixture| run_fixture(fixture, args.el, &executor))
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
fn run_fixture(fixture: &Fixture, el: Guest, executor: &Executor) -> Result<String, String> {
    let (input, expected) = el
        .io(fixture)
        .map_err(|err| format!("{} (io)\n{err:?}", fixture.name))?;
    let output = executor
        .execute(&input)
        .map_err(|err| format!("{}\n{err:?}", fixture.name))?;
    if output.get(..expected.len()) == Some(expected.as_slice()) {
        Ok(fixture.name.clone())
    } else {
        Err(format!(
            "{}\n  expected {}\n  got      {}",
            fixture.name,
            hex::encode(&expected),
            hex::encode(output.get(..expected.len()).unwrap_or(&output)),
        ))
    }
}
