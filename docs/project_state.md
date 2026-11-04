## Bulldog OS - Project State

Last Updated: November 2026

### Project Overview

Bulldog is a custom operating system written in Rust.

The project has successfully progressed beyond basic ELF loading and now includes a functioning kernel-owned process management subsystem.

Development philosophy:

- Build incrementally.
- Prove each subsystem works before adding complexity.
- Avoid introducing hidden runtime dependencies.
- Keep architecture clear and contributor-friendly.
- Maintain reproducible Windows-native builds.

---

## Current Status

### Boot Process

Working:

- UEFI bootloader loads successfully.
- Kernel loads successfully.
- ELF parsing is operational.
- HHDM memory mapping is available.
- Framebuffer initialization is functional.
- Serial logging is functional.
- Kernel reaches kernel_main() successfully.

---

### Display System

Working:

- Custom framebuffer abstraction.
- Basic screen clear operations.
- Rectangle drawing.
- Low-level framebuffer access.

---

### Logging

Working:

- Serial output available throughout boot.
- Used as primary debugging channel.
- Process subsystem verification performed through serial logging.

Known Issue:

- Build menu serial_logger toggle does not currently function correctly.

---

# Process Management

## Status: Milestone Completed

Verified through:

- /bin/hello
- /bin/goodbye
- Chained execution (hello -> goodbye)

### Process State Tracking

Implemented states:

- Ready
- Running
- Exited

Verified output:

    pid=1 state=Exited exit=Some(0)
    pid=2 state=Exited exit=Some(0)

## Verified Process Lifecycle

    create_process()
            ↓
    Ready
            ↓
    load ELF
            ↓
    update entry address
            ↓
    allocate user stack
            ↓
    Running
            ↓
    enter_user_mode()
            ↓
    sys_exit()
            ↓
    Exited

# Scheduler Development

## Current Objective

Begin scheduler architecture planning.

Planned features:

- Ready queue
- Process selection
- Scheduler data structures
- Context switch design
- Process waiting
- Blocked state management

## Current Development Rule

Before implementing multitasking:

- Process records must remain authoritative.
- Process state transitions must remain correct.
- Current PID tracking must remain correct.
- Exit handling must remain correct.
- Scheduler design must integrate with the existing process table.
