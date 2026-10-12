// File: kernel/src/vfs/init.rs

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::vfs::memfile::MemFile;
use crate::vfs::ops::{vfs_mkdir, vfs_create_file};
use crate::vfs::mount::init_mount_table;
use crate::vfs::file::FileOps;
use crate::fs::block::RamDisk;
use crate::fs::fat;

pub fn vfs_init() {
    init_mount_table();

    let _ = vfs_mkdir("/etc");
    let _ = vfs_mkdir("/etc/init");
    let _ = vfs_mkdir("/usr/bin");
    let _ = vfs_mkdir("/var/log");
    let _ = vfs_mkdir("/bin");

    let hostname = Box::new(MemFile::new(b"bulldog\n".to_vec()));
    let _ = vfs_create_file("/etc/hostname", hostname);

    let log_file = Box::new(MemFile::new(Vec::new()));
    let _ = vfs_create_file("/var/log/test_write.txt", log_file);

    // Load /bin/hello from the FAT RAM disk into a MemFile
    if let Some(bytes) = load_hello_from_fat() {
        log::info!("VFS: registered /bin/hello ({} bytes)", bytes.len());
        let hello = Box::new(MemFile::new(bytes));
        let _ = vfs_create_file("/bin/hello", hello);
    } else {
        log::error!("VFS: failed to load /bin/hello from FAT");
    }

    vfs_test_write();
}

/// Read BIN/HELLO from the FAT image already loaded by the bootloader.
fn name_starts_with_83(name: &[u8; 11], wanted: &str) -> bool {
    let w = wanted.as_bytes();
    if w.len() > 8 {
        return false;
    }
    for i in 0..w.len() {
        if (name[i] as char).to_ascii_uppercase() != (w[i] as char).to_ascii_uppercase() {
            return false;
        }
    }
    for i in w.len()..8 {
        if name[i] != b' ' {
            return false;
        }
    }
    true
}

fn load_hello_from_fat() -> Option<Vec<u8>> {
    let disk = RamDisk {
        phys_addr: crate::fat_phys_addr(),
        size: crate::fat_size(),
    };

    let bpb = fat::get_global_bpb();
    let fat_table = fat::get_global_fat();

    // Root → find BIN
    let root = fat::read_directory(&disk, bpb, fat_table, bpb.root_cluster);
    let mut bin_cluster = None;
    for e in root.iter() {
        if name_starts_with_83(&e.name, "BIN") {
            bin_cluster = Some(e.cluster());
            break;
        }
    }
    let bin_cluster = bin_cluster?;

    // BIN → find HELLO
    let bin = fat::read_directory(&disk, bpb, fat_table, bin_cluster);
    let mut hello_cluster = None;
    let mut hello_size = 0u32;
    for e in bin.iter() {
        if name_starts_with_83(&e.name, "HELLO") {
            hello_cluster = Some(e.cluster());
            hello_size = e.size;
            break;
        }
    }
    let hello_cluster = hello_cluster?;

    fat::read_file(&disk, bpb, fat_table, hello_cluster, hello_size as usize).ok()
}

pub fn vfs_test_write() {
    let mut f = MemFile::new(Vec::new());
    let n = f.write(b"abc").unwrap();
    assert_eq!(n, 3);
    f.offset = 0;
    let mut buf = [0u8; 3];
    let n = f.read(&mut buf).unwrap();
    assert_eq!(n, 3);
    assert_eq!(&buf, b"abc");
}