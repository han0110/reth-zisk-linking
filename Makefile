TOOLCHAIN ?= nightly-x86_64-unknown-linux-gnu
TARGET    := riscv64im-unknown-none-elf
LLD       ?= ld.lld
RETH_A    := reth/target/$(TARGET)/release/libreth.a
TEST_A    := test/target/$(TARGET)/release/libtest.a
ZISK_A    := zisk/target/$(TARGET)/release/libziskos_staticlib.a
CARGO     := rustup run $(TOOLCHAIN) cargo build --release --target $(TARGET) -Z build-std=core,alloc,compiler_builtins

all: build_reth build_test build_zisk link_reth link_test

build_reth:
	$(CARGO) --manifest-path reth/Cargo.toml

build_test:
	$(CARGO) --manifest-path test/Cargo.toml

build_zisk:
	RUSTFLAGS='-A explicit_builtin_cfgs_in_flags --cfg target_os="zkvm" --cfg target_vendor="zisk"' $(CARGO) --manifest-path zisk/Cargo.toml --package ziskos-staticlib

link_reth:
	$(LLD) -T linker.ld --gc-sections --no-eh-frame-hdr -o reth-zisk.elf --start-group $(ZISK_A) $(RETH_A) --end-group

link_test:
	$(LLD) -T linker.ld --gc-sections --no-eh-frame-hdr -o test-zisk.elf --start-group $(ZISK_A) $(TEST_A) --end-group

.PHONY: all build_reth build_zisk link_reth link_test
