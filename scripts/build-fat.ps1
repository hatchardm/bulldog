$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "========================================="
Write-Host "        Bulldog OS Build System"
Write-Host "========================================="
Write-Host ""
Write-Host "Select build mode:"
Write-Host "  1) Normal build"
Write-Host "  2) Build with serial logging"
Write-Host "  3) Build with syscall tests"
Write-Host "  4) Build with BOTH serial logging + syscall tests"
Write-Host ""

$choice = Read-Host "Enter choice (1-4)"

# Store ONLY feature names, not the --features flag
$features = ""

switch ($choice) {
    "1" { 
        Write-Host "[mode] Normal build selected"
        $features = ""
    }
    "2" { 
        Write-Host "[mode] Serial logging enabled"
        $features = "serial_log"
    }
    "3" { 
        Write-Host "[mode] Syscall tests enabled"
        $features = "syscall_tests"
    }
    "4" { 
        Write-Host "[mode] Serial logging + syscall tests enabled"
        $features = "serial_log,syscall_tests"
    }
    default { 
        Write-Host "[mode] Invalid choice. Defaulting to normal build."
        $features = ""
    }
}

$root = "C:\dev\bulldog"
$img = "$root\fat.img"

Write-Host ""
Write-Host "[build] Rebuilding bootloader + kernel..."
Write-Host ""

# 1. Build bootloader
cargo +nightly build -p bulldog-bootloader --release --target x86_64-unknown-uefi

# 2. Build kernel with selected features
if ($features -ne "") {
    cargo +nightly build -p kernel-bin --release --target x86_64-unknown-none -Z build-std=core,alloc --features $features
} else {
    cargo +nightly build -p kernel-bin --release --target x86_64-unknown-none -Z build-std=core,alloc
}

Write-Host ""
Write-Host "[build] Build complete."
Write-Host ""

# 3. Remove old FAT image
if (Test-Path $img) {
    Write-Host "[fat] Removing old fat.img"
    Remove-Item $img
}

# 4. Create new 64MB image
Write-Host "[fat] Creating new 64MB FAT image..."
$size = 64MB
$fs = [System.IO.File]::Create($img)
$fs.SetLength($size)
$fs.Close()

# 5. Format as FAT32
Write-Host "[fat] Formatting as FAT32..."
mformat -i $img -F ::

# 6. Create EFI directories
Write-Host "[fat] Creating EFI directory structure..."
mmd -i fat.img ::/EFI
mmd -i fat.img ::/EFI/BOOT

# 7. Copy bootloader + kernel
Write-Host "[fat] Copying bootloader + kernel..."
mcopy -o -i fat.img target\x86_64-unknown-uefi\release\bulldog-bootloader.efi ::/EFI/BOOT/BOOTX64.EFI
mcopy -o -i fat.img target\x86_64-unknown-none\release\kernel-bin ::/EFI/BOOT/kernel.elf

Write-Host ""
Write-Host "[fat] FAT image ready."
Write-Host "========================================="
Write-Host "         Bulldog OS Build Complete"
Write-Host "========================================="
