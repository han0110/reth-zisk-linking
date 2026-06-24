//! SP1 runtime shim for the zesu guest: heap symbols, log, halt.

#![no_std]

unsafe extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
    fn syscall_write(fd: u32, write_buf: *const u8, nbytes: usize);
    fn zkvm_halt(exit_code: u8) -> !;
}

/// Guest bump-allocator cursor and limit (linker symbols, resolved at link time).
#[unsafe(no_mangle)]
static mut ZKVM_HEAP_POS: *const u8 = &raw const _heap_start;
#[unsafe(no_mangle)]
static mut ZKVM_HEAP_TOP: *const u8 = &raw const _heap_end;

/// Logs to fd 2 so output never contaminates the committed public values.
#[unsafe(no_mangle)]
pub extern "C" fn zkvm_log(_level: u8, msg_ptr: *const u8, msg_len: usize) {
    unsafe {
        syscall_write(2, msg_ptr, msg_len);
        syscall_write(2, b"\n".as_ptr(), 1);
    }
}

/// Halts via libzkevm so the public values are committed first.
#[unsafe(no_mangle)]
pub extern "C" fn zkvm_exit(code: i32) -> ! {
    unsafe { zkvm_halt((code & 0xff) as u8) }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { zkvm_halt(1) }
}
