use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use spin::Mutex;

use crate::vfs::file::FileOps;
use crate::alloc::string::ToString;

/// A node in the Bulldog VFS tree.
///
/// Files are stored as Arc<Mutex<Box<dyn FileOps>>> so that all opens
/// share the same underlying file object.
pub enum VfsNode {
    File(Arc<Mutex<Box<dyn FileOps>>>),
    Directory(BTreeMap<String, VfsNode>),
    Symlink(String),
}

impl VfsNode {
    pub fn is_dir(&self) -> bool {
        matches!(self, VfsNode::Directory(_))
    }

    pub fn is_file(&self) -> bool {
        matches!(self, VfsNode::File(_))
    }

    pub fn get(&self, name: &str) -> Option<&VfsNode> {
        match self {
            VfsNode::Directory(children) => children.get(name),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut VfsNode> {
        match self {
            VfsNode::Directory(children) => children.get_mut(name),
            _ => None,
        }
    }

    pub fn mkdir(&mut self, name: &str) {
        if let VfsNode::Directory(children) = self {
            children
                .entry(name.to_string())
                .or_insert_with(|| VfsNode::Directory(BTreeMap::new()));
        }
    }

    /// The file must already be wrapped in Arc<Mutex<Box<dyn FileOps>>>.
    pub fn add_file(&mut self, name: &str, file: Arc<Mutex<Box<dyn FileOps>>>) {
        if let VfsNode::Directory(children) = self {
            children.insert(name.to_string(), VfsNode::File(file));
        }
    }
}