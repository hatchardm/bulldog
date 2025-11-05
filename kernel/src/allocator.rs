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
//use crate::{print, println};

#[global_allocator]
pub static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());


pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB


pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
  //  println!("Entered init_heap");
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

let heap_start = VirtAddr::new(HEAP_START as u64);

const MAX_BLOCK_ALIGN: u64 = 1024;
let aligned_start = VirtAddr::new(
    align_up(heap_start.as_u64().try_into().unwrap(), MAX_BLOCK_ALIGN as usize) as u64
);

let adjusted_size = (HEAP_SIZE as u64) - (aligned_start.as_u64() - heap_start.as_u64());
let aligned_start_ptr: *mut u8 = aligned_start.as_mut_ptr();

//println!(
 //   "Allocator init: aligned_start = {:#x}, adjusted_size = {}",
   // aligned_start.as_u64(),
   // adjusted_size
//);


unsafe {
    ALLOCATOR
        .lock()
        .init(aligned_start.as_u64() as usize, adjusted_size as usize);
}



//println!(
//    "Heap initialized: start = {:#x}, size = {} bytes",
    //aligned_start, adjusted_size
//);


    Ok(())
}



/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

