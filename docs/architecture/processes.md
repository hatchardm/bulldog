## Bulldog Kernel – Process Management and Scheduler Architecture

Reviewed: November 2026

This document describes Bulldog's current process-management subsystem and the future scheduler architecture that will build upon it.

Unlike earlier versions of Bulldog, process management is no longer a planned subsystem.

The kernel currently supports:

    Process creation
    PID allocation
    Process tracking
    User-mode execution
    ELF loading
    Process state transitions
    Exit code recording
    Process table dumping
    Process lifecycle management

Scheduling, context switching, and threading remain future work.

Bulldog uses a higher-half x86_64 architecture with strict separation between kernel and user execution.

---

## 1. Current Process Model

Bulldog currently implements a kernel-owned process table.

Each process record stores execution metadata and lifecycle information.

Current responsibilities include:

    PID assignment
    State tracking
    Entry-point tracking
    User stack tracking
    Exit-code tracking
    Process history retention

Process records remain in the process table after termination for diagnostics and future scheduler integration.

---

## 2. Current Process Structure

Current process records contain:

    pid
    state
    name
    entry
    stack_top
    exit_code

These fields allow Bulldog to track a process from creation until termination.

Future fields under consideration:

    page_set
    parent_pid
    kernel_stack
    scheduling_metadata

The design intentionally remains minimal until scheduler implementation begins.

---

## 3. PID Management

Bulldog currently supports:

    Automatic PID allocation
    Unique PID assignment
    Current PID tracking

The kernel stores the PID of the currently executing user process.

This allows:

    sys_exit()
    state updates
    process accounting
    future scheduling integration

Current PID tracking has been verified through user-mode execution tests.

---

## 4. Process States

Current implemented states:

    Ready
    Running
    Exited

State transitions currently verified:

    Ready
      ↓
    Running
      ↓
    Exited

These state transitions are recorded in the process table.

Future scheduler work may introduce:

    Blocked
    Sleeping
    Waiting
    Zombie

---

## 5. Process Creation

Process creation is currently implemented.

When a user program is launched:

    Process record created
    PID assigned
    Program path recorded
    Process added to table

Current implementation begins with:

    create_process()

The process initially enters the Ready state.

Additional execution information is added later during ELF loading.

---

## 6. ELF Integration

Bulldog currently creates process records during ELF execution.

The loader performs:

    ELF validation
    Program-header parsing
    Segment mapping
    Entry-point extraction
    User-stack allocation

The process record is updated with:

    Entry address
    User stack address

Stored values originate directly from:

    ELF header
    User stack allocator

This information is later used by user-mode entry and process tracking.

---

## 7. User-Mode Execution

User-mode execution is currently operational.

Bulldog supports:

    ELF loading
    User-segment mapping
    User-stack allocation
    Ring 3 entry
    Syscall handling
    Return to kernel mode

Verified programs include:

    /bin/hello
    /bin/goodbye

Current execution path:

    Load ELF
            ↓
    Map Segments
            ↓
    Allocate User Stack
            ↓
    Enter Ring 3
            ↓
    Execute User Program
            ↓
    sys_exit()
            ↓
    Return To Kernel

---

## 8. Process Lifecycle

The currently verified lifecycle is:

    create_process()
            ↓
    Ready
            ↓
    Load ELF
            ↓
    Update Entry Address
            ↓
    Allocate User Stack
            ↓
    Running
            ↓
    Enter User Mode
            ↓
    sys_exit()
            ↓
    Exited

This lifecycle has been validated through serial logging.

---

## 9. Process Exit Handling

Process termination is currently implemented.

A process may terminate through:

    sys_exit()

Current termination sequence:

    Record exit code
    Mark process exited
    Clear file descriptors
    Unmap user memory
    Apply post-exit policy
    Return to kernel

The process record remains available after termination.

This allows:

    Diagnostics
    Process-table dumping
    Future wait() support
    Future scheduler support

---

## 10. Process Chaining

Bulldog currently supports chained execution.

Verified sequence:

    /bin/hello
            ↓
    sys_exit()
            ↓
    Cleanup
            ↓
    /bin/goodbye
            ↓
    sys_exit()
            ↓
    Kernel Idle

This capability is used to validate repeated user-mode transitions and process tracking.

---

## 11. Process Table

The process table acts as the kernel's authoritative process database.

Current capabilities:

    Insert process
    Lookup process
    Update process state
    Update entry address
    Update stack address
    Store exit code
    Dump process table

Example output:

    pid=1 state=Exited exit=Some(0)
    pid=2 state=Exited exit=Some(0)

Process records intentionally persist after exit.

---

## 12. Context Switching (Future)

Full context switching is not currently implemented.

Future context switching will save and restore:

    General-purpose registers
    RIP
    RSP
    RFLAGS
    CS
    SS

Context switching will eventually support:

    Scheduler transitions
    User-mode multitasking
    Process blocking

This subsystem has not yet begun implementation.

---

## 13. Scheduling Architecture (Future)

Scheduling remains future work.

Current status:

    Process Management  -> Implemented
    Scheduler           -> Not Implemented

Planned stages:

### Stage 1

    Cooperative scheduling

Capabilities:

    Explicit yield
    Process selection
    Ready queue

### Stage 2

    Preemptive scheduling

Capabilities:

    LAPIC timer scheduling
    Context switching
    Time slicing

### Stage 3

    Advanced scheduling

Capabilities:

    Priorities
    Sleep queues
    Blocking I/O
    SMP load balancing

---

## 14. Thread Model (Future)

Threading is not currently implemented.

Planned architecture:

    Multiple threads per process
    Shared address spaces
    Separate user stacks
    Scheduler-driven execution

Thread support will be investigated after scheduler implementation stabilizes.

---

## 15. Syscall Integration

Syscalls are the only legal interface between user mode and kernel mode.

Current responsibilities include:

    Process exit
    File operations
    File descriptor management
    Memory operations

Syscalls:

    Save user context
    Execute kernel logic
    Return to user mode

Future versions may also trigger scheduling decisions.

---

## Contributor Notes

- Keep process structures minimal and well documented.
- Process records are authoritative.
- Current PID tracking must remain correct.
- Exit-code tracking must remain correct.
- Retaining exited processes is intentional.
- Scheduler work must build on the existing process subsystem.
- Context switching should remain deterministic and observable through logging.

---

## Current Status Summary

Implemented:

    PID allocation
    Current PID tracking
    Process table
    Process creation
    User-mode execution
    ELF loading
    State transitions
    Exit handling
    Process chaining
    Process-table dumping

Future Work:

    Scheduler
    Context switching
    Threading
    SMP scheduling
    Priorities
    Wait queues

---

## License

MIT or Apache 2.0 — TBD.