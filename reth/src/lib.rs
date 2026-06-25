#![no_std]

extern crate alloc;

mod rt;

use stateless_validator_reth::guest::{crypto::zkvm_interface, entrypoint};

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    rt::init_alloc();

    zkvm_interface::install_crypto();

    entrypoint::<rt::ZkVMInterfacePlatform>();

    // SP1's _start forwards this value to syscall_halt as the exit code per the
    // eth-act standard-termination spec. ZisK ignores the return value, so 0 is
    // safe for both runtimes.
    0
}
