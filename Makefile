TOOLCHAIN ?= nightly-x86_64-unknown-linux-gnu
TARGET    := riscv64im-unknown-none-elf
LLD       ?= ld.lld
RETH_A    := reth/target/$(TARGET)/release/libreth.a
ZISK_A    := zisk/target/$(TARGET)/release/libziskos_staticlib.a
ZISK_EXT_A := ziskos-staticlib-ext/target/$(TARGET)/release/libziskos_staticlib_ext.a
ZESU_O    ?= zesu.rv64im.o
ZESU_O_URL := https://github.com/Consensys/zesu/releases/download/bal-devnet-7/zesu.rv64im.o
CARGO     := rustup run $(TOOLCHAIN) cargo build --release --target $(TARGET) -Z build-std=core,alloc,compiler_builtins

all: reth zesu

reth: build_reth build_zisk link_reth

zesu: build_zisk build_zisk_ext link_zesu

build_zisk:
	$(CARGO) --manifest-path zisk/Cargo.toml --package ziskos-staticlib

build_zisk_ext:
	$(CARGO) --manifest-path ziskos-staticlib-ext/Cargo.toml

build_reth:
	$(CARGO) --manifest-path reth/Cargo.toml

link_reth:
	$(LLD) -T linker.ld --gc-sections --no-eh-frame-hdr -o reth-zisk.elf --start-group $(ZISK_A) $(RETH_A) --end-group

$(ZESU_O):
	curl -fL -o $@ $(ZESU_O_URL)

link_zesu: build_zisk_ext $(ZESU_O)
	$(LLD) -T linker.ld --gc-sections --no-eh-frame-hdr \
		-o zesu-zisk.elf --start-group $(ZISK_A) $(ZISK_EXT_A) $(ZESU_O) --end-group

.PHONY: all reth zesu build_zisk build_zisk_ext build_reth link_reth link_zesu
