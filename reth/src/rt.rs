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

unsafe extern "C" {
    fn sys_alloc_aligned(size: usize, align: usize) -> *mut u8;
}

struct CriticalSection;

critical_section::set_impl!(CriticalSection);

unsafe impl critical_section::Impl for CriticalSection {
    unsafe fn acquire() {}
    unsafe fn release(_: ()) {}
}
