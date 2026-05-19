# PoC for linking Reth with ZisK runtime

## Reth

- Implement `revm::precompile::Crypto` by [`zkvm_accelerators.h`](https://github.com/0xPolygonHermez/zisk/blob/v0.18.0/zkvm-interface/zkvm_accelerators.h) ([spec](https://github.com/eth-act/zkvm-standards/blob/main/standards/c-interface-accelerators/README.md))
- Implement IO by [`zkvm_io.h`](https://github.com/0xPolygonHermez/zisk/blob/v0.18.0/zkvm-interface/zkvm_io.h) ([spec](https://github.com/eth-act/zkvm-standards/blob/main/standards/io-interface/README.md))
- Patch all underlying `alloc::sync::Arc` to use `portable_atomic_util::Arc` ([note](https://hackmd.io/@han/compile-reth-ro-rv64im))
- Compile as a static library targeting `riscv64im-unknown-none-elf`

## ZisK

- Patch to compile `ziskos-staticlib` to `riscv64im-unknown-none-elf` ([diff](https://github.com/han0110/zisk/commit/2e267401a1cdb0046955751fc981a330517549a9#diff-6b7c6a143d6bbec6c9387291ecc9f4c6175acffe78ee91d3ead243bdceb3ad92))
- Compile with `RUSTFLAGS='-A explicit_builtin_cfgs_in_flags --cfg target_os="zkvm" --cfg target_vendor="zisk"'` (unnecessary if [`0xPolygonHermez/zisk#1007`](https://github.com/0xPolygonHermez/zisk/pull/1007) is released)
- Compile as a static library targeting `riscv64im-unknown-none-elf`

## Issues

- ZisK's runtime allocates internally (e.g. the `modexp` precompile), so the guest must share its bump-allocator state via the runtime's exported `sys_alloc_aligned`. The [standards-aligned path](https://github.com/eth-act/zkvm-standards/tree/main/standards/memory-layout-restrictions), where the guest defines its own allocator over the `_heap_start` and `_heap_end` linker symbols, bumps independent pointers over the same heap region and corrupts both sides. That path becomes viable once ZisK either stops internal allocation or exposes a standardized allocator ABI.
