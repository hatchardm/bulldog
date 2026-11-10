# Bulldog Kernel

A modern x86_64 operating system kernel written in Rust.

Bulldog is a from-scratch kernel project focused on architectural clarity, reproducible development, and incremental subsystem implementation. The project emphasizes understanding low-level operating system design while maintaining clean subsystem boundaries and comprehensive documentation.

The current architecture includes a custom UEFI bootloader, higher-half kernel execution, virtual memory management, process management, user-mode execution, FAT32 filesystem support, and a growing syscall interface.

## Current Status

### Implemented

#### Core Platform
- UEFI boot process
- Custom bootloader
- BootInfo handoff protocol
- Higher-half kernel execution
- Higher-Half Direct Map (HHDM)
- Framebuffer initialization
- Serial diagnostics

#### CPU Architecture
- GDT
- TSS
- Ring 3 support
- IST stacks
- IDT
- Exception handling
- APIC initialization
- LAPIC timer interrupts
- Interrupt handling

#### Memory Management
- Four-level paging
- OffsetPageTable integration
- Frame allocation
- Frame recycling
- LAPIC MMIO mapping
- User-page allocation
- User-stack allocation
- ELF segment mapping
- Process memory cleanup

#### Filesystem
- FAT32 support
- RAM-disk support
- Root-directory enumeration
- VFS initialization
- File-descriptor subsystem

#### Syscalls
- Syscall dispatcher
- File-descriptor operations
- Memory allocation operations
- Process exit handling
- User/kernel transitions

#### Process Management
- PID allocation
- Process table
- Process state tracking
- Exit-code tracking
- Current PID tracking
- Process cleanup
- Process chaining

#### User Mode
- Ring 3 execution
- ELF validation
- ELF loading
- ELF segment mapping
- User stacks
- Syscall entry
- Syscall return

Verified programs:
- /bin/hello
- /bin/goodbye

## Architecture Overview

UEFI Firmware -> bootloader -> boot-proto -> kernel-bin -> kernel -> kernel_main() -> kernel_init() -> User Mode / Idle Loop

## Repository Structure

- bootloader/
- boot-proto/
- kernel/
- kernel-bin/
- user/
- docs/
- scripts/

## Building

Requirements:
- Rust Nightly
- llvm-tools-preview
- QEMU
- OVMF
- mtools
- qemu-img

Install:

    rustup component add llvm-tools-preview

Build and run:

    .\\scripts\\build.ps1

Build modes:
1. Normal Build
2. Serial Logging
3. Syscall Tests
4. Serial Logging + Syscall Tests
5. Usermode Hello
6. Usermode Goodbye
7. Usermode Chain

The build pipeline builds the bootloader, kernel, user programs, FAT image, EFI System Partition, and launches QEMU.

## Development Branch

Current active development:

    feature/usermode

## Contributing

- Keep commits focused
- Justify unsafe code
- Update documentation with architectural changes
- Preserve reproducibility and observability

## License

MIT or Apache 2.0 — TBD.
