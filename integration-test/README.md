# Integration test

Fixture driven end to end tests for execution layer guests linked against the ZisK runtime. Each fixture under `fixtures/` holds a reth `StatelessInput` and the expected validation verdict. The runner converts the fixture into the guest input format, executes the linked guest ELF on ziskemu, and compares the public output against the expected bytes computed natively.

## Usage

```sh
# Build the guest ELF first, then run the suite against it.
make reth && cargo run --release -- --el reth
make zesu && cargo run --release -- --el zesu

# Useful flags, all optional.
#   --elf-path <path>   guest ELF, defaults to <repo>/{el}-zisk.elf
#   --input-dir <path>  fixture directory, defaults to <repo>/fixtures
#   --ziskemu <path>    emulator, defaults to <repo>/zisk/target/release/ziskemu
#   --filter <substr>   run only fixtures whose name contains the substring
```

## Layout

The `stateless_ssz` module is a Rust mirror of the execution-specs stateless SSZ schema built on [libssz](https://crates.io/crates/libssz), see [stateless_ssz.py at tests-zkevm@v0.4.1](https://github.com/ethereum/execution-specs/blob/tests-zkevm%40v0.4.1/src/ethereum/forks/amsterdam/stateless_ssz.py). The `zesu` module maps fixtures through that schema and computes the expected `SszStatelessValidationResult`. The `reth` module maps fixtures into the bincode input of the reth guest from [ere-guests](https://github.com/eth-act/ere-guests) and expects the SHA-256 digest of its serialized output.
