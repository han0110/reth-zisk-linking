TOOLCHAIN ?= nightly-x86_64-unknown-linux-gnu
TARGET    := riscv64im-unknown-none-elf
CARGO     := rustup run $(TOOLCHAIN) cargo build --release --target $(TARGET) -Z build-std=core,alloc,compiler_builtins
LLD       ?= ld.lld
BUILD     := build

# reth pulls the unpatched ere-guests stack, whose Arc/atomics require a target
# that advertises atomic support. The custom spec sets max-atomic-width=64 so the
# code compiles, and lower-atomic then lowers those atomics to single-core memory
# ops, leaving no atomic instructions in the linked object.
RETH_TARGET := reth/targets/$(TARGET).json
RETH_CARGO  := rustup run $(TOOLCHAIN) cargo build --release --target $(RETH_TARGET) -Z build-std=core,alloc,compiler_builtins -Z json-target-spec

RETH_RUSTFLAGS := -C passes=lower-atomic # lower-atomic strips reth's fence instructions so the SP1 executor can run it.
RETH_A     := reth/target/$(TARGET)/release/libreth.a

ZESU_O     ?= $(BUILD)/zesu.rv64im.o
ZESU_O_URL := https://github.com/Consensys/zesu/releases/download/bal-devnet-7-2026-06-24/zesu.rv64im.o

# Prebuilt Nethermind guest ELF (requires ZisK v1.0.0-alpha), emulated directly.
NETHERMIND_ELF := $(BUILD)/nethermind-zisk.elf
NETHERMIND_URL := https://github.com/NethermindEth/nethermind/releases/download/zisk-guest-r7/nethermind-guest-zisk-r7.tar.gz

SP1_A      := sp1/zkevm/sdk/libzkevm.a
SP1_LD     := lds/sp1.ld

ZISK_A     := zisk/target/$(TARGET)/release/libziskos_staticlib.a
ZISK_LD    := lds/zisk.ld

ZISK_ZESU_SHIM := shim/target/$(TARGET)/release/libzisk_zesu_shim.a
SP1_ZESU_SHIM  := shim/target/$(TARGET)/release/libsp1_zesu_shim.a
SP1_RETH_SHIM  := shim/target/$(TARGET)/release/libsp1_reth_shim.a

all: reth_sp1 zesu_sp1 reth_zisk zesu_zisk

reth_sp1:  $(BUILD)/reth-sp1.elf

zesu_sp1:  $(BUILD)/zesu-sp1.elf

reth_zisk: $(BUILD)/reth-zisk.elf

zesu_zisk: $(BUILD)/zesu-zisk.elf

nethermind_zisk: $(NETHERMIND_ELF)

clean:
	rm -rf $(BUILD)
	rm -f $(RETH_A) $(ZISK_A) $(SP1_A) $(SP1_RETH_SHIM) $(SP1_ZESU_SHIM) $(ZISK_ZESU_SHIM)

# Guest lib

$(RETH_A):
	RUSTFLAGS="$(RETH_RUSTFLAGS)" $(RETH_CARGO) --manifest-path reth/Cargo.toml

$(ZESU_O): | $(BUILD)
	curl -fL -o $@ $(ZESU_O_URL)

$(NETHERMIND_ELF): | $(BUILD)
	curl -fL $(NETHERMIND_URL) | tar -xzO nethermind > $@

# zkVM lib

$(ZISK_A):
	$(CARGO) --manifest-path zisk/Cargo.toml --package ziskos-staticlib

$(SP1_A):
	$(MAKE) -C sp1/zkevm sdk

# Shim lib

$(SP1_RETH_SHIM) $(SP1_ZESU_SHIM) $(ZISK_ZESU_SHIM) &:
	$(CARGO) --manifest-path shim/Cargo.toml

# Link

$(BUILD):
	mkdir -p $(BUILD)

$(BUILD)/reth-zisk.elf: $(ZISK_A) $(RETH_A) | $(BUILD)
	$(LLD) -T $(ZISK_LD) --gc-sections --no-eh-frame-hdr \
		-o $@ --start-group $(ZISK_A) $(RETH_A) --end-group

$(BUILD)/zesu-zisk.elf: $(ZISK_A) $(ZISK_ZESU_SHIM) $(ZESU_O) | $(BUILD)
	$(LLD) -T $(ZISK_LD) --gc-sections --no-eh-frame-hdr \
		-o $@ --start-group $(ZISK_A) $(ZISK_ZESU_SHIM) $(ZESU_O) --end-group

$(BUILD)/reth-sp1.elf: $(SP1_A) $(RETH_A) $(SP1_RETH_SHIM) | $(BUILD)
	$(LLD) -T $(SP1_LD) --gc-sections --no-eh-frame-hdr --allow-multiple-definition \
		-o $@ --start-group $(SP1_A) $(RETH_A) $(SP1_RETH_SHIM) --end-group

$(BUILD)/zesu-sp1.elf: $(SP1_A) $(SP1_ZESU_SHIM) $(ZESU_O) | $(BUILD)
	$(LLD) -T $(SP1_LD) --gc-sections --no-eh-frame-hdr --allow-multiple-definition \
		-o $@ --start-group $(SP1_A) $(SP1_ZESU_SHIM) $(ZESU_O) --end-group

# Test

test_sp1_reth:
	cargo run --release --manifest-path integration-test/Cargo.toml -- --zkvm sp1 --el reth

test_sp1_zesu:
	cargo run --release --manifest-path integration-test/Cargo.toml -- --zkvm sp1 --el zesu

test_zisk_reth:
	cargo run --release --manifest-path integration-test/Cargo.toml -- --zkvm zisk --el reth

test_zisk_zesu:
	cargo run --release --manifest-path integration-test/Cargo.toml -- --zkvm zisk --el zesu

test_zisk_nethermind: $(NETHERMIND_ELF)
	cargo run --release --manifest-path integration-test/Cargo.toml -- --zkvm zisk --el nethermind

# Test targets read ./fixtures (searched recursively). Download a set first with
# ./download-fixtures.sh {rpc-bpo2|eest-glamsterdam-devnet-5}.

.PHONY: all reth_sp1 zesu_sp1 reth_zisk zesu_zisk nethermind_zisk clean \
	test_sp1_reth test_sp1_zesu test_zisk_reth test_zisk_zesu test_zisk_nethermind
