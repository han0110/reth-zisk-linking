//! SP1 runtime shim for the reth guest: sp1-zesu-shim plus native_keccak256.

#![no_std]

// Reuse the zesu-shim surface (heap symbols, log, halt, panic handler). reth only
// references native_keccak256, but this keeps one definition of the rest.
use sp1_zesu_shim as _;

unsafe extern "C" {
    fn zkvm_keccak256(data: *const u8, len: usize, output: *mut u8) -> i32;
}

/// alloy's native-keccak hook -> SP1's zkvm_keccak256.
#[unsafe(no_mangle)]
pub extern "C" fn native_keccak256(bytes: *const u8, len: usize, output: *mut u8) {
    unsafe {
        let _ = zkvm_keccak256(bytes, len, output);
    }
}
