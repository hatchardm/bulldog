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

    // Load usermode binaries from FAT into MemFiles
    for (name_83, vfs_path) in [("HELLO", "/bin/hello"), ("GOODBYE", "/bin/goodbye")] {
        if let Some(bytes) = load_bin_from_fat(name_83) {
            log::info!("VFS: registered {} ({} bytes)", vfs_path, bytes.len());
            let file = Box::new(MemFile::new(bytes));
            let _ = vfs_create_file(vfs_path, file);
        } else {
            log::error!("VFS: failed to load {} from FAT", vfs_path);
        }
    }

    vfs_test_write();
}

/// 8.3 name match: first 8 chars, space-padded, case-insensitive.
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

/// Read `/BIN/<name_83>` from the FAT image (e.g. "HELLO", "GOODBYE").
fn load_bin_from_fat(name_83: &str) -> Option<Vec<u8>> {
    let disk = RamDisk {
        phys_addr: crate::fat_phys_addr(),
        size: crate::fat_size(),
    };

    let bpb = fat::get_global_bpb();
    let fat_table = fat::get_global_fat();

    // Root → BIN
    let root = fat::read_directory(&disk, bpb, fat_table, bpb.root_cluster);
    let mut bin_cluster = None;
    for e in root.iter() {
        if name_starts_with_83(&e.name, "BIN") {
            bin_cluster = Some(e.cluster());
            break;
        }
    }
    let bin_cluster = bin_cluster?;

    // BIN → file
    let bin = fat::read_directory(&disk, bpb, fat_table, bin_cluster);
    let mut file_cluster = None;
    let mut file_size = 0u32;
    for e in bin.iter() {
        if name_starts_with_83(&e.name, name_83) {
            file_cluster = Some(e.cluster());
            file_size = e.size;
            break;
        }
    }
    let file_cluster = file_cluster?;

    fat::read_file(&disk, bpb, fat_table, file_cluster, file_size as usize).ok()
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