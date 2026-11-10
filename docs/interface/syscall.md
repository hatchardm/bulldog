## Bulldog Kernel – Syscall Interface (Current Implementation)

Reviewed: November 2026

This document describes Bulldog's current syscall interface, including:

    ABI
    Syscall numbering
    Dispatch behavior
    Error handling
    Privilege transitions
    Harness validation

It reflects the implementation found in:

    kernel/src/syscall/mod.rs
    kernel/src/syscall/dispatcher.rs
    kernel/src/syscall/table.rs
    kernel/src/tests/syscall_harness.rs

It complements:

    syscall-entry.md
    syscall-table.md
    dispatcher.md
    privilege-switching.md

This document describes the current implementation rather than future designs.

---

## 1. Overview

Syscalls are the only supported mechanism for user-mode code to enter the kernel.

Bulldog currently uses:

    Interrupt-based syscall entry

through:

    int 0x80

User-mode code executes in:

    Ring 3

Syscall handlers execute in:

    Ring 0

The syscall subsystem provides:

    Stable ABI
    Static syscall table
    Deterministic dispatch
    Error reporting
    Harness validation

---

## 2. Syscall Entry Architecture

Syscalls enter the kernel through:

    SYSCALL_VECTOR

Current vector:

    0x80

Properties:

    DPL = 3
    User callable
    Installed by syscall subsystem
    Returns via iretq

Initialization is performed by:

    init_syscall()

The syscall handler is registered as an IDT gate and marked callable from Ring 3.

---

## 3. Syscall ABI

Bulldog currently uses the following register convention:

<table>
<tr>
<th>Register</th>
<th>Meaning</th>
</tr>

<tr>
<td>rax</td>
<td>Syscall number</td>
</tr>

<tr>
<td>rdi</td>
<td>arg0</td>
</tr>

<tr>
<td>rsi</td>
<td>arg1</td>
</tr>

<tr>
<td>rdx</td>
<td>arg2</td>
</tr>

<tr>
<td>r10</td>
<td>Reserved</td>
</tr>

<tr>
<td>r8</td>
<td>Reserved</td>
</tr>

<tr>
<td>r9</td>
<td>Reserved</td>
</tr>

</table>

Return values are placed in:

    rax

Current syscalls use up to three arguments.

---

## 4. Current Syscall Set

Implemented syscalls:

<table>
<tr>
<th>Number</th>
<th>Name</th>
</tr>

<tr>
<td>1</td>
<td>write</td>
</tr>

<tr>
<td>2</td>
<td>exit</td>
</tr>

<tr>
<td>3</td>
<td>open</td>
</tr>

<tr>
<td>4</td>
<td>read</td>
</tr>

<tr>
<td>5</td>
<td>alloc</td>
</tr>

<tr>
<td>6</td>
<td>free</td>
</tr>

<tr>
<td>7</td>
<td>close</td>
</tr>

</table>

See:

    syscall-table.md

for implementation details.

---

## 5. Dispatch Process

Current dispatch flow:

    User Program
            ↓
    int 0x80
            ↓
    syscall_handler()
            ↓
    rust_dispatch()
            ↓
    dispatch()
            ↓
    lookup()
            ↓
    Syscall Handler

Dispatch uses:

    SYSCALL_TABLE

for handler lookup.

Unknown syscall numbers 
