# Bulldog Syscall Dispatcher and Worker Model
A design document describing Bulldog’s planned syscall dispatcher, worker thread pool, and their relationship to the existing syscall ABI.

======================================================================

1. Overview
-----------

Bulldog’s current syscall path is simple and synchronous:

- User code executes `int 0x80`
- The CPU jumps to `syscall_handler`
- The handler calls `rust_dispatch(num, a0, a1, a2)`
- `rust_dispatch` looks up the handler in `SYSCALL_TABLE`
- The handler runs on the interrupt path and returns a value

This model is sufficient for early development and the syscall harness, but it has
clear limitations:

- All work happens on the interrupt path
- No structured messages
- No capability validation layer
- No separation between dispatch and execution
- No natural place for audit logging or scheduling policies

The **dispatcher + worker model** introduces a clean separation:

- The **dispatcher** validates, logs, and enqueues syscall requests
- **Worker threads** execute syscalls off the interrupt path
- Responses are returned in a structured way

This document describes that model and how it will integrate with the existing ABI.

======================================================================

2. Current Path vs Future Path
------------------------------

Current (v0.x):

    int 0x80
      → syscall_handler (assembly)
        → rust_dispatch(num, a0, a1, a2)
          → SYSCALL_TABLE[num](a0, a1, a2)
            → direct handler execution
          ← result
        ← result
      ← result in rax

Future (v1.x):

    int 0x80
      → syscall_handler (assembly)
        → rust_dispatch(num, a0, a1, a2)
          → build SyscallMessage
          → dispatcher.validate_and_enqueue(message)
          → wait for response (or fast path)
          ← response.result
        ← result
      ← result in rax

The key change: handlers no longer run directly on the interrupt path. Instead,
they are executed by worker threads that process queued SyscallMessage values.

======================================================================

3. SyscallMessage
-----------------

All syscalls will be represented as structured messages:

    struct SyscallMessage {
        num: u64,
        args: [u64; 3],
        pid: u64,
        tid: u64,
        capability: CapabilityToken,   // future
        timestamp: u64,
        id: u64,                       // unique message ID
    }

Fields:

- num: syscall number (same as current ABI)
- args: up to 3 arguments (same as current ABI)
- pid/tid: process and thread identifiers
- capability: token describing allowed operations (future)
- timestamp: time of submission (for audit/logging)
- id: unique identifier for correlating request/response

The dispatcher is responsible for constructing this message from the raw register
state and process context.

======================================================================

4. Dispatcher Responsibilities
------------------------------

The dispatcher sits between the raw syscall entry and the worker pool.

Responsibilities:

1. **Message Construction**
   - Read `rax`, `rdi`, `rsi`, `rdx`
   - Read current `pid`, `tid`
   - Attach capability token (future)
   - Attach timestamp and unique ID

2. **Validation**
   - Check syscall number is in range
   - Check syscall is implemented
   - (Future) Validate capability token
   - (Future) Enforce per‑process syscall filters

3. **Queueing**
   - Push SyscallMessage into a syscall queue
   - Choose appropriate queue (e.g., normal, high‑priority, I/O‑bound)

4. **Response Handling**
   - Wait for worker to process the message
   - Receive structured response
   - Map response into `u64` return value (or `-errno`)

5. **Logging / Auditing**
   - Log syscall start (num, pid, tid, args)
   - Log syscall completion (result, errno, duration)

The dispatcher is the central place where syscall contracts are enforced and
observed.

======================================================================

5. Worker Thread Pool
---------------------

Worker threads are kernel threads dedicated to executing syscalls.

### 5.1 Responsibilities

- Dequeue SyscallMessage from the syscall queue
- Look up the handler in SYSCALL_TABLE
- Execute the handler in a normal kernel context (not on the interrupt path)
- Produce a SyscallResponse
- Optionally apply per‑thread or per‑process security policies
- Log completion metadata

### 5.2 SyscallResponse

    struct SyscallResponse {
        id: u64,          // matches SyscallMessage.id
        result: u64,      // raw result or -(errno)
        errno: Option<Errno>,
        duration_ns: u64,
    }

The dispatcher uses `id` to match responses to waiting callers.

### 5.3 Concurrency Model

- Multiple worker threads can run in parallel
- Syscalls from different processes/threads can be interleaved
- Long‑running syscalls do not block the interrupt path
- Future: priority queues, per‑process limits, rate limiting

======================================================================

6. Integration with Existing ABI
--------------------------------

The dispatcher + worker model is designed to be **backwards‑compatible** with
the current `int 0x80` ABI and syscall table.

Key points:

- Syscall numbers do not change
- Argument registers do not change
- Return value semantics do not change
- Error encoding (`-(errno)`) does not change
- SYSCALL_TABLE remains the central handler registry

The main difference is *where* and *how* handlers are executed:

- Today: directly on the interrupt path
- Future: via SyscallMessage → dispatcher → worker → response

From the perspective of user‑mode code and the syscall harness, nothing changes.

======================================================================

7. Phased Implementation Plan
-----------------------------

To avoid destabilizing the kernel, the dispatcher and worker model will be
introduced in phases.

### Phase 1 — Internal Dispatcher Skeleton

- Implement SyscallMessage and SyscallResponse types
- Implement a basic dispatcher that:
  - constructs SyscallMessage
  - directly calls the handler (no queue, no workers)
- Add logging around dispatch

At this stage, behavior is identical to today, but the structure is in place.

### Phase 2 — Single‑Threaded Queue

- Introduce a syscall queue
- Dispatcher enqueues messages
- A single worker thread dequeues and executes handlers
- Dispatcher waits synchronously for response

This moves handler execution off the interrupt path but keeps semantics the same.

### Phase 3 — Worker Pool

- Add multiple worker threads
- Allow concurrent syscall execution
- Add basic scheduling policies (e.g., FIFO, per‑queue)

### Phase 4 — Capability Validation

- Attach CapabilityToken to SyscallMessage
- Dispatcher validates capabilities before enqueueing
- Handlers assume capabilities are pre‑validated

### Phase 5 — Advanced Policies and Audit

- Add syscall filters (per‑process allowlists)
- Add rate limiting or quotas
- Enhance logging with structured audit records

======================================================================

8. Interaction with VFS and FD Table
------------------------------------

The dispatcher and worker model does not change the semantics of:

- VFS path resolution
- FileOps and VfsFileLike
- FD allocation and reuse
- Error codes

Handlers such as `sys_open`, `sys_read`, `sys_write`, and `sys_close` continue
to operate on:

- the global FD table (for now)
- VfsFileLike wrappers
- VFS‑backed files

The only change is that these handlers will eventually run in worker threads
instead of on the interrupt path.

======================================================================

9. Future Extensions
--------------------

Once the dispatcher and worker pool are in place, Bulldog can evolve toward:

- Asynchronous syscalls (non‑blocking I/O)
- Per‑process syscall statistics and tracing
- Syscall filtering (seccomp‑like)
- Sandboxed user‑space wrapper libraries
- Hybrid fast paths (e.g., direct execution for trivial syscalls)

The dispatcher becomes the natural place to plug in these features.

======================================================================

10. Summary
-----------

The dispatcher + worker model is the next major step in Bulldog’s syscall
architecture. It:

- preserves the existing ABI
- introduces structured SyscallMessage and SyscallResponse types
- moves syscall execution off the interrupt path
- creates a central point for validation, logging, and policy
- prepares the kernel for capability‑secured, message‑based syscalls

This design bridges the gap between the current minimal syscall layer and the
long‑term capability‑based model, without breaking existing tests or future
user‑space code.

======================================================================

End of Document