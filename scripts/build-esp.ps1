# scripts/build-esp.ps1
$root = Split-Path $PSScriptRoot -Parent
$esp = Join-Path $root "esp.img"
$fat = Join-Path $root "fat.img"

Write-Host "[esp] Creating EFI System Partition..."
Remove-Item $esp -ErrorAction Ignore

# Create a 256MB raw disk
& "C:\Program Files\qemu\qemu-img.exe" create -f raw $esp 256M

Write-Host "[esp] Formatting FAT32 filesystem..."
mformat -i $esp -F ::

Write-Host "[esp] Creating EFI/BOOT..."
mmd -i $esp ::/EFI
mmd -i $esp ::/EFI/BOOT

Write-Host "[esp] Copying bootloader..."
mcopy -o -i $esp "$root\target\x86_64-unknown-uefi\release\bulldog-bootloader.efi" ::/EFI/BOOT/BOOTX64.EFI

Write-Host "[esp] Copying kernel..."
mcopy -o -i $esp "$root\target\x86_64-unknown-none\release\kernel-bin" ::/EFI/BOOT/kernel.elf

Write-Host "[esp] Copying fat.img into EFI filesystem..."
mcopy -o -i $esp $fat ::/EFI/BOOT/fat.img

Write-Host "[esp] ESP filesystem build complete."
