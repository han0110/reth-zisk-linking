# PoC for linking execution-layer guests with the ZisK runtime

This repository links a no_std reth stateless block validator (and the Zig "zesu"
executor) against zkVM runtimes that export the eth-act
[zkvm-standards](https://github.com/eth-act/zkvm-standards) C ABI, then runs them
through an emulator over fixtures. Execution only, no proving.

## Reth guest (`reth/`)

The guest is a thin platform adapter over the canonical
[`ere-guests`](https://github.com/eth-act/ere-guests) stateless validator. It
implements `ere_platform_core::Platform` as `ZkVMInterfacePlatform`, whose
`read_input`/`write_output` default impls call the zkvm-standards C ABI symbols
that both `ziskos-staticlib` and SP1's `libzkevm` export, then calls the canonical
`entrypoint`. All validation logic lives upstream in ere-guests; only the platform
binding is local.

- Input is canonical schema-prefixed SSZ; output is the sha256 digest of the SSZ
  `StatelessValidationResult`.
- The full unpatched reth/revm/alloy/stateless stack (reth v2.3.0) compiles to
  `riscv64im-unknown-none-elf` via the custom target spec
  [`reth/targets/riscv64im-unknown-none-elf.json`](reth/targets/riscv64im-unknown-none-elf.json),
  which sets `max-atomic-width: 64` so `Arc`/atomics compile. `-C passes=lower-atomic`
  then lowers them to single-core memory ops, leaving no atomic instructions in the
  object. This retires the previous portable-atomic fork stack.
- Built as a static library and linked against either runtime.

## ZisK runtime (`zisk/` submodule)

- Pinned to the upstream `v1.0.0-alpha` tag, whose transpiler natively executes the
  RISC-V A extension.
- `ziskos-staticlib` is built with `zisk-custom-alloc` so it declares no global
  allocator (the reth guest owns it); a `ForbiddenAlloc` guard turns any stray global
  allocation into a link error.
- Accelerator/precompile scratch allocations are routed to a dedicated 2 MiB `.bss`
  arena (reset per accelerator call) via the `alloc_extern` seam, so they never consume
  the guest heap.

## SP1 runtime (`sp1/` submodule)

The same reth guest links against SP1's `libzkevm` (a separate `make sp1_sdk` build)
plus `shim/sp1-reth-shim`, since libzkevm also exports the zkvm-standards C ABI.

## Integration tests (`integration-test/`)

`cargo run -- --zkvm <zisk|sp1> --el <reth|zesu|nethermind>` normalizes each fixture to
canonical schema-prefixed SSZ input bytes and the canonical SSZ
`StatelessValidationResult` output bytes, runs the input through the selected guest on
the selected backend in-process, and compares the committed output. reth output is
compared as the full sha256 digest; zesu and nethermind are compared on the
`new_payload_request_root` + `successful_validation` prefix, since those guests emit
their own chain config in the output tail.

Two fixture layouts are auto-detected per file, and directories are searched recursively
(so `--input-dir` can point at one fixture set or at `fixtures/` to run several). The
RPC fixtures carry top-level `statelessInputBytes`/`statelessOutputBytes` in `.json.zst`
files; the EEST fixtures carry the same fields per block.

Pass `--report <path>` to write a markdown pass/fail report (a `K/N passed` header plus a
table of failing fixtures and their error). CI runs each job this way, discards
stdout/stderr, and publishes the report to the GitHub job summary so each
`{el}-{zkvm} ({fixture})` pair's status is visible at a glance.

Targets: `make test_zisk_reth`, `test_sp1_reth`, `test_zisk_zesu`, `test_sp1_zesu`,
`test_zisk_nethermind`. Download a fixture set first.

### Fixture sets

`./download-fixtures.sh <name>` fetches a set into `fixtures/<name>`.

- `rpc-bpo2` pulls the `han0110/ere-guests`
  [`rpc-fixtures@v0.1.0`](https://github.com/han0110/ere-guests/releases/tag/rpc-fixtures@v0.1.0)
  RPC-derived mainnet blocks.
- `eest-glamsterdam-devnet-5` pulls the `ethereum/execution-specs`
  [`tests-zkevm@v0.4.1`](https://github.com/ethereum/execution-specs/releases/tag/tests-zkevm@v0.4.1)
  `blockchain_test` set, 23264 blocks all schema `0x0001`, so reth, zesu, and nethermind
  consume them without translation.

The CI matrix runs `{zisk, sp1} x {reth, zesu} x {rpc-bpo2, eest-glamsterdam-devnet-5}`,
plus nethermind on ZisK over both sets.

### Nethermind guest (`--el nethermind`, ZisK only)

The prebuilt
[`zisk-guest-r7`](https://github.com/NethermindEth/nethermind/releases/tag/zisk-guest-r7)
ELF (requires ZisK v1.0.0-alpha) is downloaded and emulated directly, passing all 500
fixtures on the full output. Unlike zesu, Nethermind echoes the input chain config (it
derives the active fork from the input), so the entire `StatelessValidationResult` (root,
success, chain config) matches and is compared in full. Nethermind and ere-guests both
implement execution-specs
`stateless_ssz.py`, and every SSZ container (execution payload, witness, chain config,
public keys) is byte-identical between them. The only wire difference is the 2-byte
schema prefix: Nethermind selects the payload type from the schema id (`0` =
pre-Amsterdam V3, `1` = Amsterdam V4), whereas ere-guests always emits `0x0001`. The
nethermind IO module therefore reuses the canonical SSZ body verbatim and rewrites only
the prefix, chosen from the payload variant (`ElectraFulu` -> `0`, `Gloas` -> `1`).
ere-guests' `ElectraFulu`/`Gloas` shapes match Nethermind's 4-field `NewPayloadRequest`
exactly; the pre-Electra shapes have no r7 equivalent.
