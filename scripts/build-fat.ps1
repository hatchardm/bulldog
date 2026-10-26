# scripts/build-fat.ps1
$root = Split-Path $PSScriptRoot -Parent
$fat = Join-Path $root "fat.img"

Write-Host "[fat] Creating raw FAT32 disk..."
Remove-Item $fat -ErrorAction Ignore

# Create a 64MB raw disk
& "C:\Program Files\qemu\qemu-img.exe" create -f raw $fat 64M

Write-Host "[fat] Formatting FAT32 filesystem..."
mformat -i $fat -F ::

Write-Host "[fat] Creating /bin directory..."
mmd -i $fat ::/bin

Write-Host "[fat] Copying usermode programs..."
mcopy -o -i $fat "$root\target\x86_64-unknown-none\release\hello" ::/bin/hello
mcopy -o -i $fat "$root\target\x86_64-unknown-none\release\goodbye" ::/bin/goodbye

Write-Host "[fat] FAT filesystem build complete."
