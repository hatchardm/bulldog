# Bulldog Kernel – SMP (Symmetric Multiprocessing) Architecture

This document describes Bulldog’s planned SMP architecture, including AP bring‑up,
per‑CPU structures, LAPIC configuration, and scheduling interactions.  
Bulldog does **not** currently implement SMP — this document defines the direction and
invariants that future development must follow.

It reflects the real state of the kernel today and the architectural plan for adding
multi‑CPU support.

---

## 1. Overview

Bulldog’s SMP subsystem will provide:

- Bootstrap Processor (BSP) initialization (already implemented)
- Application Processor (AP) startup (future)
- Per‑CPU LAPIC configuration
- Per‑CPU kernel stacks
- Per‑CPU TSS entries
- Per‑CPU scheduler queues
- Inter‑processor interrupts (IPIs)
- CPU‑local data structures

The design is intentionally simple and incremental.

---

## 2. Current State (BSP Only)

Today, Bulldog runs exclusively on the **Bootstrap Processor (BSP)**:

- BSP LAPIC is initialized  
- BSP LAPIC timer is active  
- BSP GDT/TSS is installed  
- BSP IDT is installed  
- BSP kernel stack + IST stacks are active  

No APs are brought online yet.

---

## 3. SMP Bring‑Up Plan

SMP bring‑up will follow the standard x86‑64 AP startup sequence:

### Step 1 — Discover CPUs

Using ACPI MADT (future):

- Enumerate LAPIC entries  
- Identify APIC IDs  
- Identify enabled CPUs  
- Record CPU count  

### Step 2 — Allocate Per‑CPU Structures

For each AP:

- Kernel stack  
- IST stacks  
- TSS  
- GDT  
- CPU‑local data  
- Scheduler queue  

These will be allocated in higher‑half memory.

### Step 3 — Send INIT IPI

BSP sends an INIT IPI to each AP:

```
APIC_ICR = INIT | TARGET_APIC_ID
```

AP enters reset state.

### Step 4 — Send SIPI (Startup IPI)

BSP sends a SIPI pointing to the AP trampoline:

```
APIC_ICR = SIPI | (trampoline_phys >> 12)
```

AP begins executing the trampoline code.

### Step 5 — AP Trampoline

The trampoline:

- Runs in real mode  
- Switches to protected mode  
- Switches to long mode  
- Jumps to AP entry point  

This code will live in a dedicated `.ap_trampoline` section.

### Step 6 — AP Initialization

Each AP performs:

- Load GDT  
- Load TSS  
- Load IDT  
- Initialize LAPIC  
- Initialize LAPIC timer  
- Switch to its per‑CPU kernel stack  
- Signal “ready” to BSP  

### Step 7 — Scheduler Integration

Once all APs are ready:

- BSP enables preemptive scheduling  
- APs begin executing runnable tasks  
- LAPIC timers drive scheduling on all CPUs  

---

## 4. Per‑CPU Data Structures

Each CPU will have a `CpuLocal` structure:

```
struct CpuLocal {
    id: u32,
    lapic_id: u32,
    kernel_stack: VirtAddr,
    tss: Tss,
    gdt: Gdt,
    scheduler_queue: SchedulerQueue,
    current_process: Option<ProcessId>,
}
```

This structure will be stored in a CPU‑local region accessed via:

- GS base register  
- Or a per‑CPU array indexed by APIC ID  

GS‑based access is preferred.

---

## 5. LAPIC Configuration (Per‑CPU)

Each CPU’s LAPIC will be configured independently:

- LAPIC base MSR read  
- LAPIC enable bit check  
- LAPIC ID verification  
- LAPIC timer setup  
- LAPIC spurious vector setup  
- LAPIC EOI handling  

The LAPIC timer will drive per‑CPU scheduling.

---

## 6. Inter‑Processor Interrupts (IPIs)

Bulldog will use IPIs for:

- TLB shootdowns  
- Scheduler wakeups  
- Cross‑CPU syscalls (future)  
- CPU halt / shutdown  
- Debugging signals  

IPIs are sent via LAPIC ICR:

```
APIC_ICR = VECTOR | DELIVERY_MODE | TARGET_APIC_ID
```

Delivery modes:

- Fixed  
- NMI  
- INIT  
- SIPI  

---

## 7. Scheduling and SMP

Bulldog’s scheduler will evolve to support SMP:

### Stage 1 — BSP‑only (current)

- No APs  
- No IPIs  
- No load balancing  

### Stage 2 — AP bring‑up

- APs join scheduler  
- Each CPU runs its own queue  
- LAPIC timers drive scheduling  

### Stage 3 — Load balancing (future)

- Work stealing  
- CPU affinity  
- NUMA awareness  

---

## 8. Memory Model

SMP requires:

- Per‑CPU kernel stacks  
- Per‑CPU IST stacks  
- Per‑CPU TSS  
- Per‑CPU GDT  
- Shared kernel heap  
- Shared frame allocator  
- Shared direct physical map  

Guard pages will be added around per‑CPU stacks.

---

## 9. Debugging SMP

SMP debugging will include:

- Logging AP startup  
- Logging LAPIC IDs  
- Logging trampoline entry  
- Logging per‑CPU initialization  
- Logging scheduler decisions  
- Logging IPIs  

Debugging SMP is notoriously difficult; Bulldog will keep logs deterministic.

---

## 10. Contributor Notes

- SMP is not implemented yet — this document defines the plan  
- AP bring‑up must be deterministic and logged  
- Per‑CPU structures must not overlap  
- LAPIC configuration must be per‑CPU  
- Scheduler must remain simple until APs are stable  
- Avoid premature optimization until SMP is functional  

---

## License

MIT or Apache 2.0 — TBD.
