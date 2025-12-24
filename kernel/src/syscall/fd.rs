// File: kernel/src/syscall/fd.rs
//! File descriptor table management for Bulldog kernel.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use log::info;
use spin::Mutex;

pub trait FileLike: Send {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        let _ = buf;
        0
    }
    fn write(&mut self, buf: &[u8]) -> usize {
        let _ = buf;
        0
    }
}

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl FileLike for Stdin {}
impl FileLike for Stdout {
    fn write(&mut self, buf: &[u8]) -> usize {
        if let Ok(s) = core::str::from_utf8(buf) {
            info!("[STDOUT] {}", s);
        }
        buf.len()
    }
}
impl FileLike for Stderr {
    fn write(&mut self, buf: &[u8]) -> usize {
        if let Ok(s) = core::str::from_utf8(buf) {
            // Keep stderr distinct in logs
            info!("[STDERR] {}", s);
        }
        buf.len()
    }
}

static FD_TABLE: Mutex<Option<BTreeMap<u64, Box<dyn FileLike + Send>>>> = Mutex::new(None);

/// Initialize FD table and seed std fds (0,1,2).
pub fn init_fd_table_with_std() {
    let mut guard = FD_TABLE.lock();
    if guard.is_none() {
        let mut map: BTreeMap<u64, Box<dyn FileLike + Send>> = BTreeMap::new();
        map.insert(0, Box::new(Stdin)  as Box<dyn FileLike + Send>);
        map.insert(1, Box::new(Stdout) as Box<dyn FileLike + Send>);
        map.insert(2, Box::new(Stderr) as Box<dyn FileLike + Send>);
        *guard = Some(map);
        info!("[FD] initialized with std fds 0=stdin,1=stdout,2=stderr");
    }
}


/// Get a locked reference to the FD table (lazy-safe if you call init first).
pub fn current_process_fd_table() -> spin::MutexGuard<'static, Option<BTreeMap<u64, Box<dyn FileLike + Send>>>> {
    FD_TABLE.lock()
}



