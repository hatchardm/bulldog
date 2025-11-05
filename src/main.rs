use std::env;

fn main() {
    let uefi_path = env::var("UEFI_PATH").expect("UEFI_PATH not set by build.rs");
    let bios_path = env::var("BIOS_PATH").expect("BIOS_PATH not set by build.rs");

    // 🧪 Sanity check: print paths
    println!("UEFI path: {}", uefi_path);
    println!("BIOS path: {}", bios_path);

    let uefi = true;


    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    cmd.arg("-machine").arg("pc");
    cmd.arg("-cpu").arg("qemu64,+apic");
    cmd.arg("-smp").arg("2");
    cmd.arg("-m").arg("512M");
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-global").arg("kvm-pit.lost_tick_policy=discard");

    if uefi {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
    } else {
        cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
    }

    // ✅ Launch QEMU
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}



