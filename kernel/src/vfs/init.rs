// File: kernel/src/vfs/init.rs

use alloc::boxed::Box;

use crate::vfs::memfile::MemFile;
use crate::vfs::ops::{vfs_mkdir, vfs_create_file};
use crate::vfs::mount::init_mount_table;
use alloc::vec::Vec;
use crate::vfs::file::FileOps;


pub fn vfs_init() {
    // Ensure mount table is initialized
    init_mount_table();

    // Create /etc directory (ignore EEXIST for now)
    let _ = vfs_mkdir("/etc");

    let _ = vfs_mkdir("/etc/init");
    let _ = vfs_mkdir("/usr/bin");
    let _ = vfs_mkdir("/var/log");

    // Create /etc/hostname backed by a MemFile
    let hostname = Box::new(MemFile::new(b"bulldog\n".to_vec()));
    let _ = vfs_create_file("/etc/hostname", hostname);

    // New: backing file for syscall write test
    let log_file = Box::new(MemFile::new(Vec::new()));
    let _ = vfs_create_file("/var/log/test_write.txt", log_file);


 // TEMPORARY: VFS write test
 vfs_test_write();

}

pub fn vfs_test_write() {
    let mut f = MemFile::new(Vec::new());

    // Write "abc"
    let n = f.write(b"abc").unwrap();
    assert_eq!(n, 3);

    // Reset offset to the start
    f.offset = 0;

    let mut buf = [0u8; 3];
    let n = f.read(&mut buf).unwrap();
    assert_eq!(n, 3);
    assert_eq!(&buf, b"abc");
}