//! Allocator subsystem root (`allocator.rs`).
//!
//! - Defines the global allocator (`ALLOCATOR`) used by the kernel.
//! - Provides `init_heap` to map heap pages and initialize the allocator.
//! - Wraps `spin::Mutex` in `Locked` for safe trait implementations.
//! - Re‑exports submodules (`fixed_size_block`, `linked_list`) for allocator strategies.

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use linked_list_allocator::LockedHeap;
use crate::allocator::fixed_size_block::FixedSizeBlockAllocator;
use crate::allocator::fixed_size_block::align_up;
use log::{info, debug};

/// Global allocator for the kernel.
/// 
/// Uses a fixed‑size block allocator wrapped in `Locked` for thread safety.
/// This is the allocator backing `alloc` crate types (e.g., `Vec`, `Box`).
#[global_allocator]
pub static ALLOCATOR: Locked<FixedSizeBlockAllocator> =
    Locked::new(FixedSizeBlockAllocator::new());

/// Fixed‑size block allocator implementation.
pub mod fixed_size_block;
/// Linked‑list allocator implementation (alternative strategy).
pub mod linked_list;

/// Virtual start address of the kernel heap.
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// Heap size in bytes (100 KiB).
pub const HEAP_SIZE: usize = 100 * 1024;

/// Initialize the kernel heap.
/// 
/// - Maps heap pages into the virtual address space.
/// - Aligns heap start to maximum block alignment.
/// - Initializes the global allocator with adjusted size.
/// 
/// Returns `Ok(())` if successful, or `MapToError` if frame allocation fails.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    info!("Entered init_heap");

    // Compute page range for heap.
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Map each heap page to a physical frame.
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    // Align heap start to maximum block alignment.
    let heap_start = VirtAddr::new(HEAP_START as u64);
    const MAX_BLOCK_ALIGN: u64 = 1024;
    let aligned_start = VirtAddr::new(
        align_up(heap_start.as_u64().try_into().unwrap(), MAX_BLOCK_ALIGN as usize) as u64,
    );

    let adjusted_size = (HEAP_SIZE as u64) - (aligned_start.as_u64() - heap_start.as_u64());

    debug!(
        "Allocator init: aligned_start = {:#x}, adjusted_size = {}",
        aligned_start.as_u64(),
        adjusted_size
    );

    // Initialize global allocator.
    unsafe {
        ALLOCATOR
            .lock()
            .init(aligned_start.as_u64() as usize, adjusted_size as usize);
    }

    debug!(
        "Heap initialized: start = {:#x}, size = {} bytes",
        aligned_start,
        adjusted_size
    );

    Ok(())
}

/// Wrapper around `spin::Mutex` to permit trait implementations.
/// 
/// Provides a simple lock/unlock interface for allocator types.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    /// Create a new `Locked` wrapper.
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    /// Acquire the lock and return a guard.
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}


