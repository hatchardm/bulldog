// File: kernel/src/vfs/resolve.rs
//! Minimal path resolver for Bulldog.
//! This now walks the mount table and VFS tree, but still returns ENOENT/ENOSYS
//! for callers. It is not yet wired into syscalls.

use alloc::string::String;
use alloc::vec::Vec;

use crate::syscall::errno::Errno;
use crate::vfs::file::{FileOps, FileResult};
use crate::vfs::mount::mount_table;
use crate::vfs::node::VfsNode;
use alloc::boxed::Box;

/// Resolve a path into a FileOps object.
///
/// Current behavior:
/// - Finds the root mount ("/").
/// - Normalizes and splits the path.
/// - Walks the VFS tree under the root mount.
/// - Always returns ENOENT or ENOSYS at the end, because we don't yet
///   know how to turn a VfsNode into a concrete FileOps instance.
///
/// This means that, effectively, resolve_path still behaves as "not implemented"
/// for all paths, but the traversal logic is now in place and ready for the VFS.
pub fn resolve_path(path: &str) -> FileResult<Box<dyn FileOps>> {
    let norm = normalize_path(path);

    // Lock the mount table and find the root mount ("/").
    let guard = mount_table();
    let root = match guard.iter().find(|m| m.path == "/") {
        Some(m) => &m.root,
        None => return Err(Errno::ENOENT),
    };

    // Special case: "/" currently has no openable object behind it.
    if norm == "/" {
        return Err(Errno::ENOSYS);
    }

    let components = split_path(&norm);

    // Walk the VFS tree starting from root.
    let mut node = root;
    for comp in components {
        match node {
            VfsNode::Directory(children) => {
                match children.get(&comp) {
                    Some(child) => node = child,
                    None => return Err(Errno::ENOENT),
                }
            }
            // Trying to descend into a non-directory node is currently unsupported.
            _ => return Err(Errno::ENOSYS),
        }
    }

    // We successfully found a node, but we don't yet have a way to
    // create a FileOps object from it. That will come in a later step.
    Err(Errno::ENOSYS)
}

/// Normalize a path like "//foo/./bar" into a stable "/foo/./bar" form.
/// For now this is very simple: ensure a single leading slash and no trailing spaces.
fn normalize_path(path: &str) -> String {
    // Trim leading slashes to avoid "//" style paths, then add a single "/".
    let mut out = String::from("/");
    out.push_str(path.trim_start_matches('/'));
    out
}

/// Split "/foo/bar" → ["foo", "bar"] as owned Strings.
fn split_path(path: &str) -> Vec<String> {
    path.trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}