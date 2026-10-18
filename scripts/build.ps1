# scripts/build.ps1
# scripts/build.ps1
$root = Split-Path $PSScriptRoot -Parent

Write-Host ""
Write-Host "========================================="
Write-Host "        Bulldog OS Build System"
Write-Host "========================================="
Write-Host ""
Write-Host "Select build mode:"
Write-Host "  1) Normal build"
Write-Host "  2) Build with serial logging"
Write-Host "  3) Build with syscall tests (EXIT_TO_IDLE off — harness continues)"
Write-Host "  4) Build with BOTH serial logging + syscall tests (EXIT_TO_IDLE off)"
Write-Host "  5) Build with usermode hello  (EXIT_TO_IDLE on — exit → kernel idle)"
Write-Host "  6) Build with usermode goodbye  (EXIT_TO_IDLE on — exit → kernel idle)"
Write-Host ""

$choice = Read-Host "Enter choice (1-6)"

# Feature names only (no --features prefix)
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
    "5" {
        Write-Host "[mode] Usermode hello enabled"
        $features = "usermode_hello"
    }

    "6" {
        Write-Host "[mode] Usermode goodbye enabled"
        $features = "usermode_goodbye"
    }
    default {
        Write-Host "[mode] Invalid choice. Defaulting to normal build."
        $features = ""
    }
}

# Helper: extra args for cargo when features are set
$featureArgs = @()
if ($features -ne "") {
    $featureArgs = @("--features", $features)
}

Write-Host "[build] Building bootloader..."
cargo +nightly build -p bulldog-bootloader --release --target x86_64-unknown-uefi

Write-Host "[build] Building kernel..."
cargo +nightly build -p kernel-bin --release --target x86_64-unknown-none `
    -Z build-std=core,alloc @featureArgs

Write-Host "[build] Building hello..."
$env:RUSTFLAGS = "-C relocation-model=static -C link-arg=--image-base=0x100000000 -C link-arg=-TC:\dev\bulldog\user\hello\linker.ld"
cargo +nightly build -p hello --release --target x86_64-unknown-none `
    -Z build-std=core,alloc

Write-Host "[build] Building goodbye..."
$env:RUSTFLAGS = "-C relocation-model=static -C link-arg=--image-base=0x100000000 -C link-arg=-TC:\dev\bulldog\user\goodbye\linker.ld"
cargo +nightly build -p goodbye --release --target x86_64-unknown-none `
    -Z build-std=core,alloc
Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue

Write-Host "[build] Building FAT filesystem..."
./scripts/build-fat.ps1

Write-Host "[build] Building EFI System Partition..."
./scripts/build-esp.ps1

Write-Host "[build] Launching QEMU..."
./scripts/qemu.ps1