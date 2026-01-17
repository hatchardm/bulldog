Bulldog Virtual File System (VFS)
=================================

Overview
--------
Bulldog’s VFS provides a simple, predictable, contributor‑friendly abstraction for kernel‑internal file operations. It is not a POSIX VFS clone. Instead, it focuses on clarity, correctness, and stability for early kernel development and future user‑space integration.

The VFS currently supports:
- hierarchical directory structure
- file creation
- path normalization
- path resolution
- shared file objects
- open/read/write/close semantics
- persistent file contents
- correct error propagation
- integration with the syscall layer

This document describes the architecture, semantics, and future direction of Bulldog’s VFS.

Core Types
----------

1. VfsNode
----------
The VFS tree is composed of VfsNode values:

    enum VfsNode {
        File(Arc<Mutex<Box<dyn FileOps>>>),
        Directory(BTreeMap<String, VfsNode>),
        Symlink(String)   // reserved for future
    }

- File nodes store a shared file object.
- Directory nodes store a map of child names to nodes.
- Symlink nodes are placeholders for future implementation.

2. FileOps Trait
----------------
All filesystem backends implement FileOps:

    trait FileOps {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Errno>;
        fn write(&mut self, buf: &[u8]) -> Result<usize, Errno>;
        fn close(&mut self) -> Result<(), Errno>;
        fn rewind(&mut self);          // reset internal offset
        fn clone_box(&self) -> Box<dyn FileOps>;
    }

FileOps is the kernel‑internal abstraction for file‑like objects.

3. MemFile
----------
The first concrete FileOps implementation:

    struct MemFile {
        data: Vec<u8>,
        offset: usize,
    }

Semantics:
- read returns bytes from current offset
- write extends file as needed
- offset advances on read/write
- rewind resets offset to 0
- clone_box duplicates the file object

Path Semantics
--------------

1. Normalization
----------------
Paths are normalized before resolution:

- leading slashes collapsed
- repeated slashes removed
- "/foo///bar" becomes "/foo/bar"

2. Components
-------------
Paths are split on "/" into components.

3. No relative paths yet
------------------------
"." and ".." are not implemented.

4. No symlink resolution yet
----------------------------
Symlink nodes exist but always return ENOSYS.

Mount Table
-----------
Bulldog currently supports a single root mount:

- mount point: "/"
- root node: a Directory

Future versions will support:
- multiple mounts
- devfs
- procfs
- ramdisk mounts

File Creation
-------------

Files are created using:

    vfs_create_file(path, Box<dyn FileOps>)

Semantics:
- intermediate directories are created automatically
- existing files are replaced
- errors:
  - EISDIR if path is "/"
  - ENOTDIR if walking into a file
  - ENOSYS for symlink traversal

Path Resolution
---------------

    resolve_path(path) -> Result<Arc<Mutex<Box<dyn FileOps>>>, Errno>

Semantics:
- walks the VFS tree from root
- returns shared file object
- errors:
  - ENOENT: missing component
  - ENOTDIR: walked into a file
  - EISDIR: final node is a directory
  - ENOSYS: symlinks not implemented

Open Semantics
--------------

Open is implemented in sys_open.

Key rules:
- each open returns a new FD
- all FDs share the same underlying file object
- the kernel calls rewind() on each open
- FD table stores a VfsFileLike wrapper

This ensures:
- persistent file contents
- each open starts at offset 0
- multiple opens do not interfere with each other’s starting position

Read Semantics
--------------

- reads return bytes from the shared file’s offset
- offset advances after read
- read returns:
  - n > 0: bytes read
  - 0: EOF

Write Semantics
---------------

- writes extend the file as needed
- offset advances after write
- writes always succeed unless:
  - file is not writable (future)
  - memory allocation fails (future)

Close Semantics
---------------

- close calls FileOps::close()
- FD table entry is removed
- underlying file object persists

Error Semantics
---------------

The VFS and syscall layer return:

- ENOENT: missing file or directory
- ENOTDIR: path component is a file
- EISDIR: attempted to open a directory as a file
- ENOSYS: symlink not implemented
- EBADF: invalid file descriptor
- EMFILE: FD table full
- EINVAL: invalid arguments

These match the syscall harness expectations.

Integration with Syscalls
-------------------------

The following syscalls interact with the VFS:

- sys_open
- sys_read
- sys_write
- sys_close

sys_open:
- resolves path
- rewinds file object
- allocates FD

sys_read/sys_write:
- operate on VfsFileLike
- delegate to FileOps

sys_close:
- removes FD entry
- calls FileOps::close()

Future Directions
-----------------

The VFS is intentionally minimal but designed for growth.

Planned enhancements:

1. Symlink resolution
2. Multiple mount points
3. devfs (device filesystem)
4. procfs (process information filesystem)
5. Per‑FD offsets instead of shared offset
6. Directory iteration
7. File permissions and metadata
8. Ramdisk filesystem backend
9. User‑space visible filesystem API

The current implementation provides a stable foundation for these features.

Summary
-------

Bulldog’s VFS is a simple, predictable, and stable subsystem that supports:

- hierarchical directories
- persistent file contents
- correct open/read/write/close semantics
- shared file objects
- clean error handling
- syscall integration

It is now fully validated by the syscall harness and ready for user‑space integration.