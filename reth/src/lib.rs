#![no_std]

extern crate alloc;

mod crypto;
mod rt;
mod stateless;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    rt::init_alloc();

    crypto::install_crypto();

    let input: &[u8] = read_input();
    let output = stateless::compute(input);
    write_output(&output);

    // SP1's __start forwards this value to syscall_halt as the exit code per the
    // eth-act standard-termination spec. ZisK ignores the return value, so 0 is
    // safe for both runtimes.
    0
}

pub fn read_input() -> &'static [u8] {
    let mut data: *const u8 = core::ptr::null();
    let mut len: usize = 0;
    unsafe { zkvm_interface::read_input(&mut data, &mut len) };
    unsafe { core::slice::from_raw_parts(data, len) }
}

pub fn write_output(output: &[u8]) {
    unsafe { zkvm_interface::write_output(output.as_ptr(), output.len()) };
}
