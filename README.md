# PoC for linking Reth with ZisK runtime

## Reth

- Implement `revm::precompile::Crypto` and `alloy_consensus::crypto::CryptoProvider` by [`zkvm_accelerators.h`](https://github.com/0xPolygonHermez/zisk/blob/v0.18.0/zkvm-interface/zkvm_accelerators.h) ([spec](https://github.com/eth-act/zkvm-standards/blob/main/standards/c-interface-accelerators/README.md))
- Implement IO by [`zkvm_io.h`](https://github.com/0xPolygonHermez/zisk/blob/v0.18.0/zkvm-interface/zkvm_io.h) ([spec](https://github.com/eth-act/zkvm-standards/blob/main/standards/io-interface/README.md))
- Patch all underlying `alloc::sync::Arc` to use `portable_atomic_util::Arc` ([note](https://hackmd.io/@han/compile-reth-ro-rv64im))
- Compile as a static library targeting `riscv64im-unknown-none-elf`

## ZisK

- Use [`feature/ziskos-staticlib-use-dedicated-scratch-space`](https://github.com/eth-act/zisk/tree/feature/ziskos-staticlib-use-dedicated-scratch-space) branch to avoid allocation in `ziskos-staticlib`
- Compile as a static library targeting `riscv64im-unknown-none-elf`
