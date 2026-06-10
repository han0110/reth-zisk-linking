#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

unsafe extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
}

static mut HEAP_POS: usize = 0;
static mut HEAP_END: usize = 0;

pub fn init_alloc() {
    unsafe { HEAP_POS = core::ptr::addr_of!(_heap_start) as usize };
    unsafe { HEAP_END = core::ptr::addr_of!(_heap_end) as usize };
}

#[global_allocator]
static ALLOC: Alloc = Alloc;

struct Alloc;

unsafe impl core::alloc::GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { alloc(layout.size(), layout.align()) }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {}

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { alloc(layout.size(), layout.align()) }
    }
}

#[inline(always)]
pub unsafe fn alloc(size: usize, align: usize) -> *mut u8 {
    let mut heap_pos = unsafe { HEAP_POS };

    let offset = heap_pos & (align - 1);
    if offset != 0 {
        heap_pos += align - offset;
    }

    let ptr = heap_pos as *mut u8;
    heap_pos += size;

    if unsafe { HEAP_END } < heap_pos {
        panic!()
    }

    unsafe { HEAP_POS = heap_pos };

    ptr
}

struct CriticalSection;

critical_section::set_impl!(CriticalSection);

unsafe impl critical_section::Impl for CriticalSection {
    unsafe fn acquire() {}
    unsafe fn release(_: ()) {}
}
