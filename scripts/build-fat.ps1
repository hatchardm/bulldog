$ErrorActionPreference = "Stop"

$root = "C:\Users\hatch\dev\bulldog"
$bootloader = "$root\bootloader\target\x86_64-unknown-uefi\debug\bulldog-bootloader.efi"
$img = "$root\fat.img"

# Remove old image
if (Test-Path $img) { Remove-Item $img }

# Create a 1.44MB file (1474560 bytes)
$size = 1474560
$fs = [System.IO.File]::Create($img)
$fs.SetLength($size)
$fs.Close()

# Format it as FAT12
mformat -i $img -f 1440 ::

# Create directories
mmd -i $img ::/EFI
mmd -i $img ::/EFI/BOOT

# Copy font file
mcopy -i $img "$root\font8x16.bin" ::

# Copy bootloader
mcopy -i $img $bootloader ::/EFI/BOOT/BOOTX64.EFI

