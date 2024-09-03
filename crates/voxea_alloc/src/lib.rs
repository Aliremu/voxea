#![feature(once_cell_get_mut)]

pub mod perf;

use std::alloc::{GlobalAlloc, Layout};

/// Custom global allocator which uses the [`MiMalloc`] allocator internally.
/// Tracks memory usage, times, etc. for regions of code
pub struct MemAllocator {
    pub(crate) inner: mimalloc::MiMalloc,
}

impl MemAllocator {
    pub const fn new() -> Self {
        Self {
            inner: mimalloc::MiMalloc,
        }
    }
}

unsafe impl GlobalAlloc for MemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // MIGHT HAVE PERFORMANCE COSTS DUE TO OVERHEAD
        // WILL FIND A BETTER SOLUTION MAYBE
        perf::alloc(layout.size());
        self.inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        perf::dealloc(layout.size());
        self.inner.dealloc(ptr, layout)
    }
}
