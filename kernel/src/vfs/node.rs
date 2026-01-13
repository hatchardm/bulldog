// File: kernel/src/vfs/node.rs
//! Core VFS node type for Bulldog.
//! This is purely additive and not yet wired into syscalls.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use crate::vfs::file::FileOps;

/// A node in the virtual filesystem tree.
pub enum VfsNode {
    /// A regular file backed by a FileOps implementation.
    File(Box<dyn FileOps>),

    /// A directory containing named children.
    Directory(BTreeMap<String, VfsNode>),

    /// A symbolic link to another path.
    Symlink(String),
}