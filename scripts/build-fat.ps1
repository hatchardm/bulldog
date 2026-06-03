$ErrorActionPreference = "Stop"

$root = "C:\dev\bulldog"
$img = "$root\fat.img"

Write-Host "[build] Rebuilding bootloader + kernel..."

# 1. Build bootloader
cargo +nightly build -p bulldog-bootloader --release --target x86_64-unknown-uefi


# 2. Build kernel
cargo +nightly build -p kernel --release --target x86_64-unknown-none -Z build-std=core,alloc

Write-Host "[build] Build complete."

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
mcopy -o -i fat.img target\x86_64-unknown-none\release\kernel ::/EFI/BOOT/kernel.elf


Write-Host "[fat] FAT image ready."


