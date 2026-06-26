#![no_std]

mod rt;

use stateless_validator_reth::guest::{crypto::zkvm_interface, entrypoint};

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    rt::init_alloc();
    zkvm_interface::install_crypto();
    entrypoint::<rt::ZkVMInterfacePlatform>();
    // SP1's _start forwards this to syscall_halt as the exit code per eth-act standard-termination.
    // ZisK ignores it, so 0 is safe.
    0
}
