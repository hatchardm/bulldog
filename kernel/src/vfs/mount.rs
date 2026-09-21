// File: kernel/src/vfs/mount.rs
//! Global mount table for Bulldog's VFS.
//! This is purely additive and not yet wired into syscalls.

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use crate::vfs::node::VfsNode;
use spin::MutexGuard;

/// A single mount point.
/// Example: path "/" → root filesystem.
pub struct MountPoint {
    pub path: String,
    pub root: VfsNode,
}

/// Global mount table.
/// For now, contains only a single root directory.
static MOUNT_TABLE: Mutex<Vec<MountPoint>> = Mutex::new(Vec::new());

/// Initialize the mount table with a single empty root directory.
/// Call this during kernel init (after heap is ready).
pub fn init_mount_table() {
    let mut guard = MOUNT_TABLE.lock();

    if !guard.is_empty() {
        return; // already initialized
    }

    guard.push(MountPoint {
        path: String::from("/"),
        root: VfsNode::Directory(Default::default()),
    });
}

/// Get a reference to the global mount table.
pub fn mount_table() -> spin::MutexGuard<'static, Vec<MountPoint>> {
    MOUNT_TABLE.lock()
}


pub fn vfs_root_mut() -> MutexGuard<'static, Vec<MountPoint>> {
    MOUNT_TABLE.lock()
}

pub fn vfs_root() -> MutexGuard<'static, Vec<MountPoint>> {
    MOUNT_TABLE.lock()
}