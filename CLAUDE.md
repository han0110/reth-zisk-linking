# CLAUDE.md

Agent context for `reth-zisk-linking`. The README is the human overview; this
file holds architecture, build recipes, encoding contracts, applied fixes, and
gotchas needed to work in the repo without re-deriving them.

## Architecture

EL guests (reth, zesu, nethermind) import undefined eth-act zkvm-standards C ABI
symbols and a zkVM runtime archive resolves them at link time. The crypto and IO
live in the RUNTIME, not the guest.

- C ABI symbols: `read_input`, `write_output`, `zkvm_keccak256`, `zkvm_sha256`,
  `zkvm_ripemd160`, `zkvm_blake2f`, `zkvm_modexp`, `zkvm_kzg_point_eval`,
  `zkvm_secp256k1_ecrecover`, `zkvm_bn254_*`, `zkvm_bls12_*`, plus heap/log/exit.
- ZisK provides them via `libziskos_staticlib.a` (built from the `zisk/` submodule
  at tag `v1.0.0-alpha`). SP1 provides them via `libzkevm.a` (built from the
  `sp1/zkevm/` submodule with the succinct toolchain).
- Guests: `reth/` builds from source; zesu is a prebuilt `.o`; nethermind is a
  prebuilt ELF (ZisK only).
- `shim/` supplies the runtime surface (heap symbols, log, halt, panic handler)
  plus hook adapters the guests don't bring themselves, in three crates:
  `sp1-reth-shim` (adds alloy's `native_keccak256` -> `zkvm_keccak256`, reusing
  `sp1-zesu-shim`'s surface), `sp1-zesu-shim`, `zisk-zesu-shim`. reth-zisk links
  no shim (the ere-guests crate + ziskos cover everything); reth-sp1 links
  `sp1-reth-shim`; zesu links its zesu shim on both backends. So the C ABI symbols
  above are imported undefined by the guests, defined by ziskos-staticlib /
  libzkevm, and bridged by `shim/` where a guest expects a differently-named hook.
- `reth/src` is just `lib.rs` + `rt.rs`. `lib.rs` (`#![no_std]`) declares `mod rt;`
  and imports `stateless_validator_reth::guest::{crypto::zkvm_interface,
  entrypoint}`; `main()` = `rt::init_alloc()` -> `zkvm_interface::install_crypto()`
  -> `entrypoint::<rt::ZkVMInterfacePlatform>()`. All stateless-validation and
  crypto logic lives in the external `stateless-validator-reth` crate (git
  `eth-act/ere-guests`, branch `han/feature/canonical-stateless-input`, feature
  `zkvm-interface`), NOT in this repo; `install_crypto` registers providers that
  call the C ABI symbols. `rt.rs` holds the bump `#[global_allocator]` (heap spans
  `_heap_start`..`_heap_end`), the `panic=abort` handler, a no-op
  `critical-section` impl, and `ZkVMInterfacePlatform` (`print` = no-op;
  `read_input`/`write_output` use the trait's C-ABI defaults).
- The old local `crypto.rs`, `stateless.rs`, and `stateless/` tree (1,139 lines,
  7 files) were dead code `lib.rs` never declared as modules, so the compiler
  excluded them entirely and they never reached the staticlib. They were a
  superseded earlier approach, replaced by the canonical crate above (crypto via
  its `zkvm_interface` providers, stateless logic via `entrypoint`), and removed
  this iteration. The change was low-risk precisely because nothing ever compiled
  them. Everything else (`integration-test`, `shim`, `rt.rs`, `Makefile`) is
  referenced and live.

## Build and link

- `make <el>_<zkvm>` builds `build/<el>-<zkvm>.elf`. Targets: `reth_zisk`,
  `zesu_zisk`, `reth_sp1`, `zesu_sp1`, `nethermind_zisk`.
- Each link is `ld.lld -T lds/<zkvm>.ld --gc-sections --no-eh-frame-hdr` over the
  archives inside a `--start-group ... --end-group`. SP1 links add
  `--allow-multiple-definition` (the shim redefines symbols libzkevm also exports);
  ZisK links don't.
- `reth/` compiles the unpatched reth/revm/alloy stack to a custom target
  (`reth/targets/riscv64im-unknown-none-elf.json`, `max-atomic-width: 64`) with
  `-Z build-std` and `RUSTFLAGS="-C passes=lower-atomic"`, which lowers atomics to
  single-core memory ops, leaving no atomic or fence instructions.
- libzkevm and ziskos-staticlib have no auto-rebuild prereqs in the Makefile;
  after editing their source, force a rebuild before relinking:
  - libzkevm: `make -C sp1/zkevm sdk` (needs `~/.sp1/bin` on PATH).
  - ziskos: `rustup run nightly cargo build --release --target
    riscv64im-unknown-none-elf -Z build-std=core,alloc,compiler_builtins
    --manifest-path zisk/Cargo.toml --package ziskos-staticlib`.
  Then `make <el>_<zkvm>` relinks.
- The harness is `cargo build --release --manifest-path integration-test/Cargo.toml`.
  It emulates in-process by linking the published `sp1-core-executor` crate and the
  `zisk-core`/`ziskemu` git crates (tag `v1.0.0-alpha`); no shell `ziskemu` binary or
  `zisk/` checkout is needed to run, only to build `libziskos_staticlib.a`.

## Fixtures and harness

- `./download-fixtures.sh {rpc-bpo2|eest-glamsterdam-devnet-5}` populates
  `fixtures/<name>`. rpc-bpo2 = 20 real mainnet blocks (`.json.zst`, top-level
  `statelessInputBytes`/`statelessOutputBytes`). eest = execution-spec
  blockchain_tests (same fields per block), all schema `0x0001`.
- `integration-test/` normalizes both layouts to canonical schema-prefixed SSZ,
  runs a guest on a backend, and compares the committed output. reth is compared
  on the full sha256 digest; zesu and nethermind on the
  `new_payload_request_root + successful_validation` prefix (they emit their own
  chain-config tail).
- Flags: `--zkvm`, `--el`, `--elf-path`, `--input-dir`, `--filter` (substring on
  file path), `--limit`, `--report <path>` (markdown report, written before bail).

## Encoding contracts (load-bearing)

The C ABI carries the EIP-2537 / EVM byte encoding that the canonical guest and
ziskos use. The zkcrypto `bls12_381` crate (inside SP1's libzkevm) uses a
different native encoding, so libzkevm needs an adapter:

- G1 = `x || y` (48 BE each). G2 = `x.c0 || x.c1 || y.c0 || y.c1` (c0 first). The
  crate serializes G2 as c1-first, so libzkevm must swap the c0/c1 halves.
- Point at infinity = all zeros. The crate uses a `0x40` flag bit in byte 0, and
  masks the top 3 flag bits on decode, so libzkevm must reject any coordinate with
  those bits set (>= p) and translate all-zeros <-> the crate's flag.
- `map_fp2_to_g2(0)` is the SWU u=0 exceptional input; the crate returns infinity
  but EIP-2537 wants a fixed finite point (hardcoded in libzkevm).
- ripemd output = `[12 zero bytes || 20 hash bytes]` (left-padded, first 12 zero).
- blake2 (and every ZisK precompile) requires operand pointers 8-byte aligned;
  the guest must provide aligned buffers.

## Applied fixes (under review)

- SP1 BLS: `sp1/zkevm/libzkevm/src/precompile/bls12_381.rs` gained EVM<->crate
  adapters, a canonical-flag guard, the `map_fp2(0)` constant, a pairing
  identity-skip, and the map_fp2 c0/c1 read order. Conformance harness
  (`tests/conformance/{bls,support}.rs`) updated to the EVM ABI; full geth +
  wycheproof suites pass on host. eip2537: 539 -> 0.
- SP1 ripemd: `hash.rs` writes the digest to `out[12..]` (was `out[..20]`); the
  ABI header comment was corrected. reth-sp1 EEST: 62 -> 16.
- ZisK BLS: cherry-picked 3 commits from `0xPolygonHermez/zisk#1140` (subgroup
  checks, map-to-curve, minor fixes) onto the `zisk/` submodule. eip2537: 22 -> 0.
- The `sp1/` changes are uncommitted working-tree edits; the `zisk/` changes are
  local unpushed cherry-pick commits. Nothing in the parent repo is committed.

## EL vs zkVM attribution (current EEST failures)

Both zkVM runtimes are correct. Every remaining failure is an EL (guest) issue:

- reth: 16 on both backends, identical to the host (no zkVM) result. reth/revm vs
  fixture divergences (14 eip7610, 1 eip8037 stale fixture, 1 eip8025).
- zesu (prebuilt, not fixable here): ripemd (46, both backends) reads the hash
  from the wrong 20 bytes; blake2 (47, ZisK only) passes a 4-byte-aligned buffer
  that ZisK's precompile ABI rejects (SP1 software blake2 tolerates it); 2 EVM
  quirks.
- nethermind (prebuilt ELF, not relinked by this repo): 303 BLS + 242 EVM
  divergences + 69 emulator panics, baked into the r7 build. Rebuilding it against
  PR #1140's ziskos would clear the BLS and most panics.

## Gotchas

- `rm` is blocked in this environment; use `git clean -fdx <path>` for cleanup.
- Never run git add/commit/stash/push without explicit instruction. Submodule
  changes are left for the user to review and commit.
- zesu and nethermind are prebuilt binaries; their bugs can only be flagged, not
  fixed here.
- `docs/` is gitignored (local notes only).
- The ZisK blake2 alignment and BLS panics are caught per-fixture (the zisk path
  uses `catch_unwind`); the SP1 path halts on guest faults without a Rust panic.
- `clippy` flags `large_enum_variant` on `zkvm::Executor` (`Sp1(Sp1Pool)` is far
  larger than `Zisk(ZiskExecutor)`). Pre-existing, not introduced by minimization;
  `Box`ing the `Sp1Pool` variant clears it. Don't treat it as a regression.
- This iteration ran an LoC-minimization pass that intentionally stripped doc and
  section comments (Makefile section headers, `integration-test` fn docs and
  `Fixture` field docs, `zisk.rs` module doc). The sparse-comment style is
  deliberate; match it and do not re-pad removed comments.
