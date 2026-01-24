#![no_std]

extern crate alloc;

pub mod errno;
mod raw;
pub mod wrappers;

pub use errno::{Errno, SysResult};
pub use wrappers::{write, read, open, close, exit};