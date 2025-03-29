#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::fmt::Write;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::serial;

pub struct BumpAllocator;

pub static mut HEAP_START: usize = 0x0;
pub static HEAP_OFFSET: AtomicUsize = AtomicUsize::new(0);
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Calculate the required alignment
        let align = layout.align();
        let size = layout.size();
        
        // Get the current allocation position
        let current_offset = HEAP_OFFSET.load(Ordering::Relaxed);
        
        // Calculate the aligned allocation address
        let aligned_offset = (current_offset + align - 1) & !(align - 1);
        
        // Check if we have enough memory left
        if aligned_offset + size > HEAP_SIZE {
            writeln!(serial(), "Out of memory: offset={}, size={}", aligned_offset, size).unwrap();
            return null_mut();
        }
        
        // Update the offset atomically
        HEAP_OFFSET.store(aligned_offset + size, Ordering::Relaxed);
        
        // Calculate and return the allocation address
        let allocation_address = unsafe { HEAP_START + aligned_offset };
        writeln!(serial(), "Allocated {} bytes at {:p} with align {}", 
                 size, allocation_address as *mut u8, align).unwrap();
        
        allocation_address as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        writeln!(serial(), "dealloc was called at {ptr:?}").unwrap();
    }
}

pub fn init_heap(offset: usize) {
    unsafe {
        HEAP_START = offset;
        HEAP_OFFSET.store(0, Ordering::Relaxed);
        
        // Zero out the heap region for safety
        for i in 0..HEAP_SIZE {
            *((HEAP_START + i) as *mut u8) = 0;
        }
        
        writeln!(serial(), "Heap initialized at {:p} with size {}", 
                 HEAP_START as *mut u8, HEAP_SIZE).unwrap();
    }
}