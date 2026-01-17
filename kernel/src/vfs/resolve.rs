// File: kernel/src/vfs/resolve.rs
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;

use crate::syscall::errno::Errno;
use crate::vfs::file::FileOps;
use crate::vfs::mount::mount_table;
use crate::vfs::node::VfsNode;

/// Resolve a path into a shared FileOps handle.
///
/// Returns:
///   Ok(Arc<Mutex<Box<dyn FileOps>>>) → file found
///   Err(ENOENT)                      → not found
///   Err(EISDIR)                      → final node is a directory
///   Err(ENOSYS)                      → symlinks not supported yet
pub fn resolve_path(path: &str) -> Result<Arc<Mutex<Box<dyn FileOps>>>, Errno> {
    let norm = normalize_path(path);

    let guard = mount_table();
    let root_mount = guard
        .iter()
        .find(|m| m.path == "/")
        .ok_or(Errno::ENOENT)?;

    let mut node = &root_mount.root;

    if norm == "/" {
        return Err(Errno::EISDIR);
    }

    let components = split_path(&norm);

    for comp in components {
        match node {
            VfsNode::Directory(children) => {
                match children.get(&comp) {
                    Some(child) => node = child,
                    None => return Err(Errno::ENOENT),
                }
            }
            VfsNode::File(_) => return Err(Errno::ENOTDIR),
            VfsNode::Symlink(_) => return Err(Errno::ENOSYS),
        }
    }

    match node {
        VfsNode::File(f) => Ok(f.clone()), // clone Arc, not file
        VfsNode::Directory(_) => Err(Errno::EISDIR),
        VfsNode::Symlink(_) => Err(Errno::ENOSYS),
    }
}

/// Normalize a path like "//foo/bar" → "/foo/bar"
fn normalize_path(path: &str) -> String {
    let mut out = String::from("/");
    out.push_str(path.trim_start_matches('/'));
    out
}

/// Split "/foo/bar" → ["foo", "bar"]
fn split_path(path: &str) -> Vec<String> {
    path.trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}