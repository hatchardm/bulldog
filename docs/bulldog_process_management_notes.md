# Bulldog Process Management Notes

Last Updated: November 2026

## Purpose

This document records the current design and implementation status of the Bulldog process-management subsystem.

The goal is to preserve architectural decisions, implementation details, and future plans so that development can resume quickly after long pauses.

---

## Current Status

Status: COMPLETE (Phase 1)

The process-management subsystem has been implemented and verified.

Verified through:

- /bin/hello
- /bin/goodbye
- hello -> goodbye chained execution

The kernel now owns and maintains process state throughout the complete lifecycle of a process.

---

## Current Process Structure

Current fields:

- pid
- state
- name
- entry
- stack_top
- exit_code

Future fields under consideration:

- page_set
- parent_pid
- kernel_stack
- user_context
- scheduling_metadata

---

## Process States

Implemented:

- Ready
- Running
- Exited

Potential future states:

- Blocked
- Sleeping
- Waiting
- Zombie

---

## PID Management

Implemented:

- Automatic PID allocation
- Unique PID assignment
- Current PID tracking

Current PID is tracked separately from the process table and is used by sys_exit() to identify the currently executing process.

---

## Process Lifecycle

Verified lifecycle:

Ready
  ↓
Running
  ↓
Exited

Actual flow:

create_process()
        ↓
Ready
        ↓
load ELF
        ↓
store entry address
        ↓
allocate user stack
        ↓
set current pid
        ↓
Running
        ↓
enter_user_mode()
        ↓
sys_exit()
        ↓
Exited

---

## ELF Integration

Implemented:

- Process creation during ELF loading
- Entry-point extraction from hdr.e_entry
- Process record update after ELF validation
- User-stack tracking

The earlier hdr scope problem has been resolved by obtaining the ELF entry point within the loader and passing it into the process-management subsystem.

---

## Exit Handling

Implemented:

- Current PID lookup
- Exit-code recording
- State transition to Exited
- File-descriptor cleanup
- User-memory cleanup
- Process history retention

Verified output:

pid=1 state=Exited exit=Some(0)
pid=2 state=Exited exit=Some(0)

---

## Validated Behaviors

Successfully demonstrated:

- Process creation
- Process tracking
- Process state transitions
- Exit code recording
- ELF metadata tracking
- User stack tracking
- Process-table dumping
- Sequential process execution
- Process chaining

---

## Design Decisions

### Why Exited Processes Remain In The Table

Exited processes are intentionally retained.

Reasons:

- Simplifies debugging
- Preserves process history
- Allows later implementation of wait()
- Allows later implementation of waitpid()
- Makes scheduler development easier

### Why No Scheduler Yet

The process subsystem was completed first so the scheduler can be built on top of a verified process database.

This reduces complexity and isolates failures.

---

## Deferred Features

Not implemented yet:

- Ready queue
- Scheduler
- Context switching
- Timer-driven scheduling
- Blocking
- Waiting
- Parent/child processes
- SMP scheduling
- Priorities
- CPU affinity

These belong to future scheduler work.

---

## Next Milestone

Scheduler Architecture.

Expected topics:

- Ready queue design
- Process selection strategy
- Scheduler data structures
- Context-switch architecture
- Timer integration
- Process state transitions beyond Running/Exited

---

## Developer Notes

Current subsystem status can be considered stable.

The process table has been verified using the hello -> goodbye chain test and is suitable as the foundation for future scheduler implementation.
