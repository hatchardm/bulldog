# Bulldog Kernel – SMP and Per‑CPU Architecture

This document describes Bulldog’s planned symmetric multiprocessing (SMP) architecture,
including CPU discovery, AP startup, per‑CPU data structures, and interrupt routing. While
full SMP support is a future milestone, this document defines the architectural direction
and invariants that contributors should follow as the subsystem evolves.

Bulldog targets the `x86_64-bulldog` architecture and uses the APIC model for interrupt
delivery and CPU coordination.

---

## Overview

Bulldog’s SMP subsystem is designed around the following goals:

- Deterministic multi‑core bring‑up  
- Per‑CPU kernel stacks and data regions  
- Clean separation between BSP (Bootstrap Processor) and APs (Application Processors)  
- LAPIC‑based interrupt routing  
- Scalable scheduling and context switching  
- Contributor‑friendly debugging and logging  

This document outlines the architecture that future multi‑core scheduling and parallel
execution features will build upon.

---

## CPU Roles

### Bootstrap Processor (BSP)

The BSP:

- Starts execution from the bootloader  
- Initializes paging and higher‑half mapping  
- Sets up memory management  
- Configures the IDT, GDT, and TSS  
- Initializes the LAPIC  
- Brings up Application Processors (APs)  

### Application Processors (APs)

APs:

- Start in real mode or wait-for-SIPI state  
- Receive INIT + SIPI signals from the BSP  
- Jump into a trampoline entry point  
- Enter long mode  
- Switch to the higher‑half kernel  
- Initialize their per‑CPU structures  
- Join the scheduler  

---

## AP Startup Flow

The AP startup sequence follows the standard x86 APIC protocol:

```
BSP
  ↓
Send INIT IPI
  ↓
Send SIPI (Startup IPI) with trampoline address
  ↓
AP executes trampoline code
  ↓
AP enters long mode
  ↓
AP loads GDT/TSS
  ↓
AP switches to higher‑half kernel
  ↓
AP initializes per‑CPU data + stack
  ↓
AP signals "ready"
```

The trampoline code is identity‑mapped and must reside below 1 MiB.

---

## Per‑CPU Data

Each CPU receives its own per‑CPU region containing:

- Kernel stack  
- CPU‑local scheduler state  
- LAPIC ID  
- Current process/thread pointer (future)  
- CPU‑local scratch space  
- Interrupt statistics (future)  

Example layout:

```
[ Guard Page ]
[ Kernel Stack ]
[ Per-CPU Data ]
```

Per‑CPU regions are placed in a reserved higher‑half range for easy indexing.

---

## LAPIC and Interrupt Routing

Each CPU has a Local APIC (LAPIC) responsible for:

- Timer interrupts  
- Inter‑processor interrupts (IPIs)  
- Error reporting  
- Local interrupt delivery  

Bulldog uses LAPIC timer interrupts for:

- Preemptive scheduling (future)  
- Timekeeping  
- CPU‑local timers  

### Inter‑Processor Interrupts (IPIs)

IPIs will be used for:

- TLB shootdowns  
- Scheduler wakeups  
- Cross‑CPU coordination  
- Kernel debugging (future)  

---

## GDT and TSS per CPU

Each CPU receives:

- Its own TSS  
- Its own kernel stack pointer  
- A shared GDT (initially)  

The TSS contains:

- RSP0 (kernel stack)  
- IST entries for critical exceptions  

This ensures safe privilege transitions on every CPU.

---

## Scheduler Interaction

Once SMP is enabled:

- Each CPU runs the scheduler independently  
- Load balancing (future) distributes tasks across CPUs  
- Context switching remains per‑CPU  
- Processes may migrate between CPUs (future)  

Early SMP will use:

- Cooperative scheduling on all CPUs  
- Later: preemptive scheduling via LAPIC timers  

---

## Memory Ordering and Synchronization

Bulldog will use:

- Spinlocks for early SMP  
- Atomic operations for shared structures  
- Memory barriers where required  
- Per‑CPU caches to reduce contention  

Future enhancements:

- Ticket locks  
- MCS locks  
- RCU (Read‑Copy‑Update)  

---

## Debugging SMP

SMP introduces new debugging challenges:

- Race conditions  
- Deadlocks  
- Stack corruption on APs  
- Incorrect APIC routing  
- Misconfigured TSS or GDT  

Bulldog will include:

- Per‑CPU logging buffers  
- AP startup logs  
- LAPIC ID verification  
- IPI tracing  

---

## Future SMP Enhancements

Planned improvements:

- CPU hotplug (future)  
- NUMA awareness  
- Per‑CPU slab allocators  
- Cross‑CPU syscall dispatch  
- Parallel kernel subsystems  

---

## Contributor Notes

- Keep SMP code minimal and deterministic  
- Document all per‑CPU structures and invariants  
- Ensure AP startup code remains identity‑mapped  
- Validate LAPIC IDs before enabling interrupts  
- Avoid global locks where possible  
- Maintain compatibility with the scheduler and process model  

---

## License

MIT or Apache 2.0 — to be determined.
