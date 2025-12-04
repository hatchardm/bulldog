//! Syscall module root
//! Re-exports dispatcher and stubs

pub mod dispatcher;
pub mod stubs;

pub use dispatcher::{
    init_syscall,
    syscall_handler,
    dispatch,
    syscall_entry,
    SYSCALL_VECTOR,   // re-export so external code can use crate::syscall::SYSCALL_VECTOR
};

pub use stubs::{SYS_WRITE, SYS_EXIT};



