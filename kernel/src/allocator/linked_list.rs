//! Linked-list allocator.
//!
//! Provides a dynamic allocator that manages free regions in a linked list.
//! Each free region is represented by a `ListNode` storing its size and a pointer
//! to the next free region.
//!
//! - Allocations search the list for a suitable region (`find_region`).
//! - Regions are split when partially used, with the remainder re-added as free.
//! - Deallocation reinserts the freed region back into the list.
//!
//! Safety notes:
//! - `init(heap_start, heap_size)` must be called once with a valid, unused heap region.
//! - `add_free_region` writes `ListNode` metadata directly into the freed memory.

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

/// Node representing a free region in the linked list.
struct ListNode {
    /// Size of the free region in bytes.
    size: usize,
    /// Pointer to the next free region.
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    /// Create a new list node with given size.
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    /// Return the start address of this region.
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    /// Return the end address of this region.
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// Linked-list allocator.
/// Maintains a head node pointing to the list of free regions.
pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    /// Create an empty allocator (requires `init` before use).
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// # Safety
    /// - Caller must guarantee that `heap_start..heap_start+heap_size` is valid and unused.
    /// - Must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    /// Add a free region to the front of the list.
    ///
    /// # Safety
    /// - Caller must ensure `addr..addr+size` is valid and writable.
    /// - Region must be large enough to hold a `ListNode`.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }

    /// Find a suitable free region for allocation.
    ///
    /// - Searches the list for a region large enough for `size` and `align`.
    /// - Removes the region from the list if found.
    ///
    /// Returns `(region, alloc_start)` on success.
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }

    /// Check if a region can satisfy an allocation of `size` and `align`.
    ///
    /// Returns the allocation start address on success.
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(()); // region too small
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            return Err(()); // remainder too small to hold a ListNode
        }

        Ok(alloc_start)
    }

    /// Adjust layout so allocated region can store a `ListNode`.
    ///
    /// Returns `(size, align)` adjusted values.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    /// Allocate memory for `layout`.
    ///
    /// - Adjusts layout to ensure space for `ListNode`.
    /// - Finds a suitable region, splits it if necessary, and returns pointer.
    /// - Returns null if no region is available.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    /// Deallocate memory at `ptr` for `layout`.
    ///
    /// - Adjusts layout to ensure freed region can store a `ListNode`.
    /// - Reinserts region into free list.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}
