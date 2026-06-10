//! Runtime extension to libziskos_staticlib.a for guests that expect host
//! provided symbols beyond the standard zkVM C interface.
//!
//! Guests such as the zesu rv64im object resolve their IO and accelerator
//! symbols from libziskos_staticlib.a directly. The remaining runtime
//! symbols are provided here. The heap variables ZKVM_HEAP_POS and
//! ZKVM_HEAP_TOP are data cells whose initial contents are the addresses of
//! the _heap_start and _heap_end linker script symbols, resolved through
//! relocations when the final ELF is linked. The emulator loads the
//! initialized data image, so the guest bump allocator observes valid heap
//! bounds without any runtime initialization code.

#![no_std]

unsafe extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
    fn sys_write(fd: u32, write_ptr: *const u8, nbytes: usize);
}

/// Bump allocator cursor consumed by the guest. The cell starts at the heap
/// bottom and the guest advances it on every allocation.
#[unsafe(no_mangle)]
static mut ZKVM_HEAP_POS: *const u8 = &raw const _heap_start;

/// Exclusive upper bound of the guest heap.
#[unsafe(no_mangle)]
static mut ZKVM_HEAP_TOP: *const u8 = &raw const _heap_end;

/// Forwards a guest log message to the ZisK UART through sys_write.
///
/// The level argument is accepted for interface compatibility and is not
/// encoded into the emitted bytes. A trailing newline keeps consecutive
/// messages readable on the emulator console.
#[unsafe(no_mangle)]
pub extern "C" fn zkvm_log(_level: u8, msg_ptr: *const u8, msg_len: usize) {
    unsafe {
        sys_write(1, msg_ptr, msg_len);
        sys_write(1, b"\n".as_ptr(), 1);
    }
}

/// Terminates the guest through the ZisK exit ecall with a7 set to 93.
///
/// The exit code is placed in a0 for diagnostic visibility. The trailing
/// jump keeps the function diverging on hosts that ignore the ecall.
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
