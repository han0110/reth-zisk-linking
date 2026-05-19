TOOLCHAIN ?= nightly-x86_64-unknown-linux-gnu
TARGET    := riscv64im-unknown-none-elf
SYSROOT   := $(shell rustup run $(TOOLCHAIN) rustc --print sysroot)
LLD       := $(SYSROOT)/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld
RETH_A    := reth/target/$(TARGET)/release/libreth.a
ZISK_A    := zisk/target/$(TARGET)/release/libziskos_staticlib.a
CARGO     := rustup run $(TOOLCHAIN) cargo build --release --target $(TARGET) -Z build-std=core,alloc,compiler_builtins

all: build_zisk build_reth link

build_reth:
	$(CARGO) --manifest-path reth/Cargo.toml

build_zisk:
	RUSTFLAGS='-A explicit_builtin_cfgs_in_flags --cfg target_os="zkvm" --cfg target_vendor="zisk"' $(CARGO) --manifest-path zisk/Cargo.toml --package ziskos-staticlib

link:
	$(LLD) -flavor gnu -T linker.ld --gc-sections --no-eh-frame-hdr -o reth-zisk.elf --start-group $(ZISK_A) $(RETH_A) --end-group

.PHONY: all build_reth build_zisk link
