// File: kernel/src/syscall/mod.rs

//! Syscall module root
//! Re-exports dispatcher, stubs, and individual syscall modules

pub mod dispatcher;
pub mod stubs;
pub mod table;
pub mod errno;

// Dedicated syscall modules
pub mod write;
pub mod exit;
pub mod open;
pub mod read;
pub mod fd;

pub use dispatcher::{
    init_syscall,
    syscall_handler,
    dispatch,
    SYSCALL_VECTOR,
};

// Re-export syscall numbers
pub use stubs::{SYS_WRITE, SYS_EXIT, SYS_OPEN, SYS_READ};

// Re-export syscall functions
pub use write::sys_write;
pub use exit::sys_exit;
pub use open::sys_open;
pub use read::sys_read;


