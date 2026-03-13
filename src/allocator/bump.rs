use crate::allocator::{Locked, align_up};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.heap_end = start + size;
        self.next = self.heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        let start = align_up(allocator.next, layout.align());
        let end = match start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if end > allocator.heap_end {
            ptr::null_mut()
        } else {
            allocator.next = end;
            allocator.allocations += 1;
            start as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        allocator.allocations -= 1;
        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}
