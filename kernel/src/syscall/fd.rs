// File: kernel/src/syscall/fd.rs
//! File descriptor table for Bulldog kernel.
//! Stores FdEntry objects which wrap FileLike trait objects.
//! Provides fd_alloc, fd_get, fd_close APIs for syscalls and VFS.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;

use spin::Mutex;

use crate::syscall::errno::Errno;
use crate::syscall::filelike::FileLike;
use crate::syscall::stubs::Stdout;

/// Maximum file descriptor number.
/// Usable range is [3, MAX_FD].
pub const MAX_FD: u64 = 64;

/// A single file descriptor entry.
/// Wraps a FileLike object plus metadata.
pub struct FdEntry {
    pub file: Box<dyn FileLike + Send>,
    pub flags: u64,
    pub offset: usize,
}

/// Per‑process FD table.
/// Maps fd → FdEntry.
pub type FdTable = BTreeMap<u64, FdEntry>;

/// Global FD table for the current process.
/// (Later this will be per‑process.)
static FD_TABLE: Mutex<Option<FdTable>> = Mutex::new(None);

/// Called from kernel_init() AFTER heap initialization.
/// Creates FD table and installs stdout on FD 1.
pub fn init_fd_table_with_std() {
    let mut guard = FD_TABLE.lock();

    if guard.is_some() {
        return; // already initialized
    }

    let mut table = BTreeMap::new();

    // FD 1 = stdout
    table.insert(
        1,
        FdEntry {
            file: Box::new(Stdout),
            flags: 0,
            offset: 0,
        },
    );

    *guard = Some(table);
}

/// Get the FD table.
/// Panics if called before init_fd_table_with_std().
pub fn current_process_fd_table() -> spin::MutexGuard<'static, Option<FdTable>> {
    let guard = FD_TABLE.lock();

    if guard.is_none() {
        panic!("FD table accessed before initialization");
    }

    guard
}

/// Allocate the lowest available FD >= 3.
pub fn fd_alloc(entry: FdEntry) -> Result<u64, Errno> {
    let mut guard = current_process_fd_table();
    let table = guard.as_mut().unwrap();

    let mut fd = 3u64;
    while fd <= MAX_FD {
        if !table.contains_key(&fd) {
            table.insert(fd, entry);
            return Ok(fd);
        }
        fd += 1;
    }

    Err(Errno::EMFILE)
}

/// Get a mutable reference to an FD entry.
pub fn fd_get(fd: u64) -> Result<&'static mut FdEntry, Errno> {
    let mut guard = current_process_fd_table();
    let table = guard.as_mut().unwrap();

    let entry_ptr: *mut FdEntry = match table.get_mut(&fd) {
        Some(e) => e,
        None => return Err(Errno::EBADF),
    };

    // SAFETY:
    // We leak the guard and return a &'static mut reference.
    // This is safe because Bulldog is single‑threaded for now.
    Ok(unsafe { &mut *entry_ptr })
}

/// Close an FD and remove it from the table.
pub fn fd_close(fd: u64) -> Result<(), Errno> {
    if fd < 3 {
        return Err(Errno::EBADF);
    }

    let mut guard = current_process_fd_table();
    let table = guard.as_mut().unwrap();

    match table.remove(&fd) {
        Some(mut entry) => entry.file.close(),
        None => Err(Errno::EBADF),
    }
}

/// Clear the FD table (used by sys_exit).
pub fn fd_clear_all() {
    let mut guard = FD_TABLE.lock();
    *guard = None;
}



