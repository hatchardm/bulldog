// File: kernel/src/vfs/ops.rs
//! Basic VFS mutation helpers: create files and directories.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use crate::syscall::errno::Errno;
use crate::vfs::file::FileOps;
use crate::vfs::mount::mount_table;
use crate::vfs::node::VfsNode;

/// Create a directory at the given absolute path.
/// Example: vfs_mkdir("/etc")
pub fn vfs_mkdir(path: &str) -> Result<(), Errno> {
    let components = split_path(path);
    if components.is_empty() {
        return Err(Errno::EINVAL);
    }

    let mut guard = mount_table();
    let root = guard
        .iter_mut()
        .find(|m| m.path == "/")
        .ok_or(Errno::ENOENT)?;

    let mut node = &mut root.root;

    for comp in components {
        match node {
            VfsNode::Directory(children) => {
                node = children.entry(comp).or_insert_with(|| {
                    VfsNode::Directory(BTreeMap::new())
                });
            }
            _ => return Err(Errno::ENOTDIR),
        }
    }

    Ok(())
}

/// Create a file at the given absolute path.
/// Example: vfs_create_file("/hello.txt", MemFile::new(b"hi".to_vec()))
pub fn vfs_create_file(path: &str, file: Box<dyn FileOps>) -> Result<(), Errno> {
    let mut components = split_path(path);
    if components.is_empty() {
        return Err(Errno::EINVAL);
    }

    let filename = components.pop().unwrap();

    let mut guard = mount_table();
    let root = guard
        .iter_mut()
        .find(|m| m.path == "/")
        .ok_or(Errno::ENOENT)?;

    let mut node = &mut root.root;

    // Walk to parent directory
    for comp in components {
        match node {
            VfsNode::Directory(children) => {
                node = children.entry(comp).or_insert_with(|| {
                    VfsNode::Directory(BTreeMap::new())
                });
            }
            _ => return Err(Errno::ENOTDIR),
        }
    }

    // Insert the file
    match node {
        VfsNode::Directory(children) => {
            if children.contains_key(&filename) {
                return Err(Errno::EEXIST);
            }
            children.insert(filename, VfsNode::File(file));
            Ok(())
        }
        _ => Err(Errno::ENOTDIR),
    }
}

/// Split "/foo/bar" → ["foo", "bar"]
fn split_path(path: &str) -> Vec<String> {
    path.trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}