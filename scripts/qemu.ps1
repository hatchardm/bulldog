# scripts/qemu.ps1
$root = Split-Path $PSScriptRoot -Parent

$OvmfCode = Join-Path $root "OVMF_CODE.fd"
$OvmfVars = Join-Path $root "OVMF_VARS.fd"
$EspImg   = Join-Path $root "esp.img"

Write-Host "Booting from ESP: $EspImg"

& "C:\Program Files\qemu\qemu-system-x86_64.exe" `
    -m 512M `
    -drive "if=pflash,format=raw,readonly=on,file=$OvmfCode" `
    -drive "if=pflash,format=raw,file=$OvmfVars" `
    -drive "format=raw,file=$EspImg" `
    -device qemu-xhci `
    -device usb-kbd `
    -serial file:kernel-serial.log
