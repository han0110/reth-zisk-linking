#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
static ALLOC: Alloc = Alloc;

struct Alloc;

unsafe impl core::alloc::GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { sys_alloc_aligned(layout.size(), layout.align()) }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {}

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { sys_alloc_aligned(layout.size(), layout.align()) }
    }
}

// FIXME: ZisK's runtime allocates internally (e.g. the modexp precompile), so
//        the guest must share its bump-allocator state via the runtime's
//        exported `sys_alloc_aligned`.
//        The standards-aligned path (guest defines its own allocator over
//        `_heap_start` and `_heap_end` linker symbols) bumps independent
//        pointers over the same heap region and corrupts both sides.
//        That path becomes viable once ZisK either stops internal allocation or
//        exposes a standardized allocator ABI.
unsafe extern "C" {
    fn sys_alloc_aligned(size: usize, align: usize) -> *mut u8;
}

struct CriticalSection;

critical_section::set_impl!(CriticalSection);

unsafe impl critical_section::Impl for CriticalSection {
    unsafe fn acquire() {}
    unsafe fn release(_: ()) {}
}
