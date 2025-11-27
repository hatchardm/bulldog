//! Fixed-size block allocator.
//!
//! Provides a small-block allocator backed by free lists for common power-of-two
//! sizes, with a fallback linked-list allocator for larger or uncommon layouts.
//!
//! - Fast path: allocate from per-size free lists (8–4096 bytes).
//! - Fallback: delegate to `linked_list_allocator::LockedHeap`.
//! - Global usage: wrapped by `Locked<FixedSizeBlockAllocator>` to implement `GlobalAlloc`.
//!
//! Safety notes:
//! - `init(heap_start, heap_size)` must be called once with a valid, unused heap region.
//! - `add_region` carves the heap into block-size-aligned free lists; the caller must
//!   ensure the region is exclusively owned by the allocator.

use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};
use linked_list_allocator::LockedHeap;

/// Singly linked list node representing a free block of a given size class.
#[repr(C)]
pub struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// Allocation error type (for potential future use).
#[derive(Debug)]
pub struct AllocError;

/// Supported block sizes (power-of-two).
/// Each size also serves as the block alignment requirement.
///
/// Rationale:
/// - Power-of-two alignments simplify `align_up` math.
/// - Covers common small allocations (`Vec`, `Box`, `String`, small structs).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

/// Choose the free-list index for the given layout.
/// Returns `Some(index)` if a suitable size class exists, otherwise `None`.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

/// Fixed-size block allocator with per-size free lists and fallback allocator.
pub struct FixedSizeBlockAllocator {
    /// Free-list heads for each size class in `BLOCK_SIZES`.
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    /// Fallback allocator for requests that don't fit a size class.
    fallback_allocator: LockedHeap,
}

impl FixedSizeBlockAllocator {
    /// Create an empty allocator (requires `init` before use).
    pub const fn new() -> Self {
        const NONE: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [NONE; BLOCK_SIZES.len()],
            fallback_allocator: LockedHeap::empty(),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// - Aligns the heap start to satisfy stricter layout requirements.
    /// - Initializes the fallback allocator with the aligned region.
    /// - Carves the region into per-size-class free lists by calling `add_region`.
    ///
    /// Safety:
    /// - `heap_start..heap_start+heap_size` must be a valid, unused, exclusively owned region.
    /// - Must be called exactly once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        // Align heap start to at least 128 bytes (covers common alignments).
        let aligned_start = align_up(heap_start, 128);
        let adjusted_size = heap_size.saturating_sub(aligned_start.saturating_sub(heap_start));

        // Initialize fallback allocator with aligned region.
        self.fallback_allocator.lock().init(aligned_start, adjusted_size);

        // Populate per-size-class free lists from the same aligned region.
        for &block_size in BLOCK_SIZES {
            self.add_region(aligned_start, adjusted_size, block_size);
        }
    }

    /// Allocate using the fallback allocator.
    ///
    /// Used when a suitable fixed-size class is unavailable or exhausted.
    pub fn fallback_alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.fallback_allocator.alloc(layout) };
        if ptr.is_null() {
            ptr::null_mut()
        } else {
            ptr
        }
    }

    /// Add a region of memory to a size-class free list.
    ///
    /// - Walks the region in `block_size` chunks.
    /// - Each chunk becomes a `ListNode` in the free list for that size class.
    ///
    /// Safety:
    /// - Caller guarantees that `heap_start..heap_start+heap_size` is valid and writable.
    /// - Region must not overlap with other allocator uses.
    unsafe fn add_region(&mut self, heap_start: usize, heap_size: usize, block_size: usize) {
        assert!(block_size.is_power_of_two());

        let aligned_start = align_up(heap_start, block_size);
        let end = heap_start.saturating_add(heap_size);

        let mut current = aligned_start;

        while current.saturating_add(block_size) <= end {
            let node = current as *mut ListNode;

            let index = Self::list_index_for(block_size);
            let prev_head = self.list_heads[index].take();
            (*node).next = prev_head;
            self.list_heads[index] = Some(&mut *node);

            current = current.saturating_add(block_size);
        }
    }

    /// Return the index for the exact `block_size`.
    fn list_index_for(block_size: usize) -> usize {
        BLOCK_SIZES
            .iter()
            .position(|&s| s == block_size)
            .expect("Invalid block size")
    }
}

/// Align `addr` up to `align` (power-of-two).
pub fn align_up(addr: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    /// Allocate memory for `layout`.
    ///
    /// Strategy:
    /// - If `layout` fits a size class, pop from the corresponding free list.
    /// - Otherwise, or if the list is empty, delegate to the fallback allocator.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => match allocator.list_heads[index].take() {
                Some(node) => {
                    allocator.list_heads[index] = node.next.take();
                    node as *mut ListNode as *mut u8
                }
                None => {
                    let block_size = BLOCK_SIZES[index];
                    let layout = Layout::from_size_align(block_size, block_size).unwrap();
                    FixedSizeBlockAllocator::fallback_alloc(&allocator, layout)
                }
            },
            None => FixedSizeBlockAllocator::fallback_alloc(&allocator, layout),
        }
    }

    /// Deallocate memory at `ptr` for `layout`.
    ///
    /// Strategy:
    /// - If `layout` fits a size class, push the block back onto that free list.
    /// - Otherwise, delegate to the fallback allocator.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // Ensure we can store a ListNode in this block.
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.dealloc(ptr.as_ptr(), layout);
            }
        }
    }
}

