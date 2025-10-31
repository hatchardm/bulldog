use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{
    mem,
    ptr::{self, NonNull},
};
use crate::{print, println};
use linked_list_allocator::LockedHeap;



#[repr(C)]
pub struct ListNode {
    next: Option<&'static mut ListNode>,
}

// At the top of fixed_size_block.rs
#[derive(Debug)]
pub struct AllocError;


/// The block sizes to use.
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

/// Choose an appropriate block size for the given layout.
///
/// Returns an index into the `BLOCK_SIZES` array.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    let index = BLOCK_SIZES.iter().position(|&s| s >= required_block_size);
   // println!("list_index: layout={:?} → index={:?}", layout, index);
    index
}




pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: LockedHeap,
}


impl FixedSizeBlockAllocator {
    /// Creates an empty FixedSizeBlockAllocator.
    pub const fn new() -> Self {
        const NONE: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [NONE; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::LockedHeap::empty(),

        }
    }


    

    /// Initialize the allocator with the given heap bounds.
    ///
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
   pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
    println!("FixedSizeBlockAllocator::init called with heap_start={:#x}, size={}", heap_start, heap_size);
    // Align heap start to satisfy stricter layout requirements
    let aligned_start = align_up(heap_start, 128); // or 64 if you prefer
    let adjusted_size = heap_size - (aligned_start - heap_start);

    // Initialize fallback allocator with aligned region
    unsafe {
    self.fallback_allocator.lock().init(aligned_start, adjusted_size);
}


    // Add regions for fixed-size blocks
    for &block_size in BLOCK_SIZES {
     //   println!("Adding region for block size: {}", block_size);
        self.add_region(aligned_start, adjusted_size, block_size);
    }
}



    /// Allocates using the fallback allocator.
pub fn fallback_alloc(&self, layout: Layout) -> *mut u8 {
    let ptr = unsafe { self.fallback_allocator.alloc(layout) };


    if ptr.is_null() {
        println!("Fallback alloc failed: layout={:?}", layout);
        ptr::null_mut()
    } else {
        println!("Fallback alloc success: layout={:?}, ptr={:p}", layout, ptr);
        ptr
    }
}











  unsafe fn add_region(&mut self, heap_start: usize, heap_size: usize, block_size: usize) {
    assert!(block_size.is_power_of_two());

    let aligned_start = align_up(heap_start, block_size);
    let end = heap_start + heap_size;

    // ✅ Insert the log here
    //Debugging the parameters
    //println!(
   //     "add_region: block_size={}, aligned_start={:#x}, end={:#x}, usable={}",
   //     block_size,
   //     aligned_start,
    //    end,
     //   end.saturating_sub(aligned_start)
   // );

    let mut current = aligned_start;
    let mut count = 0;

    while current + block_size <= end {
        let node = current as *mut ListNode;

        let index = Self::list_index_for(block_size);
        let prev_head = self.list_heads[index].take();
        (*node).next = prev_head;
        self.list_heads[index] = Some(&mut *node);

        count += 1;
        current += block_size;
    }

   // println!("Added {} blocks for size {}", count, block_size);
}



    


    fn list_index_for(block_size: usize) -> usize {
        BLOCK_SIZES.iter().position(|&s| s == block_size).expect("Invalid block size")
    }
}

 pub fn align_up(addr: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
      //   println!("GlobalAlloc::alloc called");
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

            None => FixedSizeBlockAllocator::fallback_alloc(&allocator, layout)
,
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
                let ptr = NonNull::new(ptr).unwrap();
                unsafe {
    unsafe {
    allocator.fallback_allocator.dealloc(ptr.as_ptr(), layout);
}

}

            }
        }
    }
}
