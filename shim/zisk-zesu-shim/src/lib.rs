//! ZisK runtime shim for the zesu guest: heap symbols, log, exit.

#![no_std]

unsafe extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
    fn sys_write(fd: u32, write_ptr: *const u8, nbytes: usize);
}

/// Guest bump-allocator cursor and limit (linker symbols, resolved at link time).
#[unsafe(no_mangle)]
static mut ZKVM_HEAP_POS: *const u8 = &raw const _heap_start;
#[unsafe(no_mangle)]
static mut ZKVM_HEAP_TOP: *const u8 = &raw const _heap_end;

/// Logs a guest message to the ZisK console.
#[unsafe(no_mangle)]
pub extern "C" fn zkvm_log(_level: u8, msg_ptr: *const u8, msg_len: usize) {
    unsafe {
        sys_write(1, msg_ptr, msg_len);
        sys_write(1, b"\n".as_ptr(), 1);
    }
}

/// Exits via the ZisK ecall (a7=93).
#[unsafe(no_mangle)]
pub extern "C" fn zkvm_exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "1:",
            "ecall",
            "j 1b",
            in("a7") 93u64,
            in("a0") code,
            options(noreturn),
        )
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
