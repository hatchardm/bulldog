use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    let bin_dir = out_dir.join("bin");
    fs::create_dir_all(&bin_dir).unwrap(); // ✅ Ensure bin/ exists

    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel).create_disk_image(&uefi_path).unwrap();

    let bios_path = bin_dir.join("bootloader-x86_64-bios.img");
    bootloader::BiosBoot::new(&kernel).create_disk_image(&bios_path).unwrap();

    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}

