// File: kernel/src/syscall/mod.rs

//! Syscall module root
//! Re-exports dispatcher and stubs

pub mod dispatcher;
pub mod stubs;
pub mod table;
pub mod errno;

// New dedicated syscall modules
pub mod write;
pub mod exit;
pub mod open;   

pub use dispatcher::{
    init_syscall,
    syscall_handler,
    dispatch,
    SYSCALL_VECTOR,
};

pub use stubs::{SYS_WRITE, SYS_EXIT, SYS_OPEN};
pub use write::sys_write;
pub use exit::sys_exit;
pub use open::sys_open;

