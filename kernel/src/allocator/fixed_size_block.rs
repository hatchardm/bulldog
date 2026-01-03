use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};
use linked_list_allocator::LockedHeap;

pub struct ListNode {
    next: Option<&'static mut ListNode>,
}


const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: LockedHeap,
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const NONE: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [NONE; BLOCK_SIZES.len()],
            fallback_allocator: LockedHeap::empty(),
        }
    }

    /// New initializer that takes two non-overlapping regions:
    ///
    /// - fixed-size region: carved into blocks for all size classes
    /// - fallback region: given wholly to LockedHeap
    pub unsafe fn init_with_regions(
        &mut self,
        fixed_start: usize,
        fixed_size: usize,
        fallback_start: usize,
        fallback_size: usize,
    ) {
        // init fallback first
        // This region is never touched by fixed-size lists.
        // Safety: caller guarantees non-overlap.
        self.fallback_allocator.lock().init(fallback_start, fallback_size);

        // Now carve the fixed-size region into non-overlapping blocks.
        self.populate_fixed_lists(fixed_start, fixed_size);
    }

    unsafe fn populate_fixed_lists(&mut self, region_start: usize, region_size: usize) {
        // Simple strategy: walk from small to large block sizes,
        // carving as many blocks as fit for each class from the remaining region.
        let mut current = region_start;
        let end = region_start.saturating_add(region_size);

        for &block_size in BLOCK_SIZES {
            if current >= end {
                break;
            }

            let aligned = align_up(current, block_size);
            if aligned >= end {
                break;
            }

            let mut addr = aligned;
            while addr.saturating_add(block_size) <= end {
                let node = addr as *mut ListNode;

                let index = Self::list_index_for(block_size);
                let prev_head = self.list_heads[index].take();
                (*node).next = prev_head;
                self.list_heads[index] = Some(&mut *node);

                addr = addr.saturating_add(block_size);
            }

            current = addr;
        }
    }

    fn list_index_for(block_size: usize) -> usize {
        BLOCK_SIZES
            .iter()
            .position(|&s| s == block_size)
            .expect("Invalid block size")
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.fallback_allocator.alloc(layout) };
        if ptr.is_null() {
            ptr::null_mut()
        } else {
            ptr
        }
    }

    fn fallback_dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        unsafe {
            self.fallback_allocator.dealloc(ptr.as_ptr(), layout);
        }
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
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
                    allocator.fallback_alloc(layout)
                }
            },
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                allocator.fallback_dealloc(ptr, layout);
            }
        }
    }
}


