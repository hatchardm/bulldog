// File: kernel/src/vfs/ops.rs

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use spin::Mutex;

use crate::syscall::errno::Errno;
use crate::vfs::mount::mount_table;
use crate::vfs::node::VfsNode;
use crate::vfs::file::FileOps;

/// Normalize a path like "//foo/bar" → "/foo/bar"
fn normalize_path(path: &str) -> String {
    let mut out = String::from("/");
    out.push_str(path.trim_start_matches('/'));
    out
}

/// Split "/foo/bar" → ["foo", "bar"]
fn split_path(path: &str) -> alloc::vec::Vec<String> {
    path.trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Create a directory (and any missing parents) at `path`.
pub fn vfs_mkdir(path: &str) -> Result<(), Errno> {
    let norm = normalize_path(path);

    if norm == "/" {
        return Ok(());
    }

    let components = split_path(&norm);

    let mut guard = mount_table();
    let root_mount = guard
        .iter_mut()
        .find(|m| m.path == "/")
        .ok_or(Errno::ENOENT)?;

    let mut node = &mut root_mount.root;

    for comp in components {
        match node {
            VfsNode::Directory(children) => {
                node = children
                    .entry(comp.clone())
                    .or_insert_with(|| VfsNode::Directory(BTreeMap::new()));
            }
            VfsNode::File(_) => return Err(Errno::ENOTDIR),
            VfsNode::Symlink(_) => return Err(Errno::ENOSYS),
        }
    }

    Ok(())
}

/// Create or replace a file at `path`.
///
/// Files are stored as Arc<Mutex<Box<dyn FileOps>>> so that all opens
/// share the same underlying file object.
pub fn vfs_create_file(path: &str, file: Box<dyn FileOps>) -> Result<(), Errno> {
    let norm = normalize_path(path);

    if norm == "/" {
        return Err(Errno::EISDIR);
    }

    let mut components = split_path(&norm);

    let file_name = match components.pop() {
        Some(name) => name,
        None => return Err(Errno::EINVAL),
    };

    let mut guard = mount_table();
    let root_mount = guard
        .iter_mut()
        .find(|m| m.path == "/")
        .ok_or(Errno::ENOENT)?;

    let mut node = &mut root_mount.root;

    for comp in components {
        match node {
            VfsNode::Directory(children) => {
                node = children
                    .entry(comp.clone())
                    .or_insert_with(|| VfsNode::Directory(BTreeMap::new()));
            }
            VfsNode::File(_) => return Err(Errno::ENOTDIR),
            VfsNode::Symlink(_) => return Err(Errno::ENOSYS),
        }
    }

    // Wrap the file in Arc<Mutex<Box<dyn FileOps>>>
    let shared = Arc::new(Mutex::new(file));

    match node {
        VfsNode::Directory(children) => {
            children.insert(file_name, VfsNode::File(shared));
            Ok(())
        }
        VfsNode::File(_) => Err(Errno::ENOTDIR),
        VfsNode::Symlink(_) => Err(Errno::ENOSYS),
    }
}