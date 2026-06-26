# zkEVM PoC

A proof of concept that links Ethereum execution-layer (EL) guests against zkVM
runtimes through the eth-act [zkvm-standards](https://github.com/eth-act/zkvm-standards)
C ABI, then runs them through emulators over test fixtures. Execution only, no proving.

## How it works

- EL guests import undefined `zkvm_*` C ABI symbols (crypto precompiles, IO).
- A zkVM runtime archive provides those symbols at link time.
- The linked ELF is emulated over fixtures, and the committed output is compared
  against the expected result.

## Components

| Piece | Role |
| --- | --- |
| `reth/` | EL guest: a thin adapter over the canonical eth-act stateless validator |
| zesu | EL guest: a prebuilt Consensys executor (downloaded `.o`) |
| nethermind | EL guest: a prebuilt ELF (ZisK only, downloaded) |
| ZisK (`zisk/`) | zkVM runtime + emulator, pinned to `v1.0.0-alpha` |
| SP1 (`sp1/`) | zkVM runtime (`libzkevm`), built with the succinct toolchain |
| `integration-test/` | Harness: normalizes fixtures, runs a guest on a backend, compares output |

## Quick start

```sh
# Build a guest linked against a backend
make reth_zisk          # also: zesu_zisk, reth_sp1, zesu_sp1, nethermind_zisk

# Download a fixture set
./download-fixtures.sh rpc-bpo2                  # 20 real mainnet blocks
./download-fixtures.sh eest-glamsterdam-devnet-5 # 23264 execution-spec blocks

# Run the harness
make test_zisk_reth     # also: test_sp1_reth, test_zisk_zesu, test_sp1_zesu, test_zisk_nethermind
```

Each run accepts `--report <path>` for a markdown pass/fail report; CI publishes
it to the GitHub job summary.

## Current status

All combinations pass the 20 real-mainnet `rpc-bpo2` blocks. EEST results below.

| EL x zkVM | EEST fail / 23264 | rpc-bpo2 | Remaining failures |
| --- | --- | --- | --- |
| reth x ZisK | 16 | 20 / 20 | reth/fixture divergences (host-confirmed) |
| reth x SP1 | 16 | 20 / 20 | the same 16 reth/fixture divergences |
| zesu x SP1 | 48 | 20 / 20 | zesu binary: ripemd layout |
| zesu x ZisK | 99 | 20 / 20 | zesu binary: ripemd layout + blake2 alignment |
| nethermind x ZisK | 614 | 20 / 20 | nethermind prebuilt build: BLS + EVM divergences |

Every remaining EEST failure is an EL (guest) issue, not a zkVM issue. reth, the
canonical guest, fails only the same 16 fixtures on both backends, and those 16
are also rejected by the host with no zkVM. The zesu and nethermind failures are
non-conformances in their prebuilt binaries.

See `CLAUDE.md` for architecture, build recipes, and the encoding contracts.
