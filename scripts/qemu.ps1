$Root = Split-Path $PSScriptRoot -Parent
$Qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"

$OvmfCode = Join-Path $Root "OVMF_CODE.fd"
$OvmfVars = Join-Path $Root "OVMF_VARS.fd"
$FatImg   = Join-Path $Root "fat.img"

$DriveCode = "if=pflash,format=raw,readonly=on,file=$OvmfCode"
$DriveVars = "if=pflash,format=raw,file=$OvmfVars"
$DriveFat  = "format=raw,file=$FatImg"

Write-Host "DriveCode: $DriveCode"
Write-Host "DriveVars: $DriveVars"
Write-Host "DriveFat:  $DriveFat"

& $Qemu `
    -drive $DriveCode `
    -drive $DriveVars `
    -drive $DriveFat `
    -serial stdio
