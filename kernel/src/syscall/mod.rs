// File: kernel/src/syscall/mod.rs

//! Syscall module root
//! Re-exports dispatcher and stubs

pub mod dispatcher;
pub mod stubs;
pub mod table;

pub use dispatcher::{
    init_syscall,
    syscall_handler,
    dispatch,
    syscall_entry,
    SYSCALL_VECTOR,
};

pub use stubs::{SYS_WRITE, SYS_EXIT};
