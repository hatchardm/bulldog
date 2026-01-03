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
pub mod alloc;
pub mod free;
pub mod close;


pub use dispatcher::{
    init_syscall,
    syscall_handler,
    dispatch,
    SYSCALL_VECTOR,
};

// Re-export syscall numbers
pub use stubs::{
    SYS_WRITE,
    SYS_EXIT,
    SYS_OPEN,
    SYS_READ,
    SYS_ALLOC,
    SYS_FREE,
    SYS_CLOSE,
};

// Re-export syscall functions (ABI trampolines only)
pub use write::sys_write;
pub use exit::sys_exit;
pub use open::sys_open;
pub use read::sys_read;
pub use alloc::sys_alloc_trampoline;
pub use free::sys_free_trampoline;
pub use close::sys_close;




