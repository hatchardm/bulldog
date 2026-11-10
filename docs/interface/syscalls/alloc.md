## Bulldog Kernel – alloc Syscall

Reviewed: November 2026

This document describes Bulldog's current memory allocation syscall.

The alloc syscall allocates memory using the kernel heap allocator and returns a pointer to the allocated region.

Related files:

    kernel/src/syscall/alloc.rs
    free.md
    syscall.md
    syscall-table.md

---

## 1. Purpose

The alloc syscall provides dynamic memory allocation for user-mode code.

Current syscall number:

    5

The allocation is serviced through the kernel allocator subsystem.

---

## 2. Prototype

```text
alloc(size: usize) -> *mut u8
```

*rguments:

    size
       *Number of bytes to allocate

*eturn value:

    Pointer to*allocated memory on success

    N*gative errno value on failure

---*
## 3. Implementation*
Current implementation:

```*ust
sys_alloc(size)
```

Allocatio**uses:

```rust*alloc::alloc::*lloc()
```

with*

```rust
Layout::from_size_align(*ize* 8)
```

Current*alignment:

    8 bytes

---

## *. Successful Allocation

On succes* the syscall returns:

    Non-zer* pointer

The returned value is th* base*address of the allocated memory re*ion.

Example:

```text
ptr = allo*(64)
```

Result:

```text
*ointer returned
```

The*syscall harness validates successf*l allocation behavior.

---

## *. Validation Rules

Current*validation:

###*Size Must Be Non-Z*ro

The syscall rejects*

```text
size =*0
```

Result*

```text
EINVAL
```

###*Layout Must Be Valid

Invalid layo*ts are rejected.

*esult:

```text
EINVAL
```

---

*# 6. Error Handling

Current error**

### EINVAL

Returned*when:

    size is zero
    layout*creation fails
*   allocation request is invalid

*##*ENOMEM

Returned when:

    alloca*ion fails

Return values are encod*d using the syscall errno mechanis*.

---

## 7. Harness Coverage

Th* syscall harness currently*validates:

### Successful*allocation

```text
alloc(64)
``*

Expected:

```text
Valid*pointer
```

### Zero-size allocat*on

```*ext
alloc(0)
```

Expected:

```te*t
EINVAL
```

### Extremely large *llocation

```text
*lloc(very*large_size)
```

Expected:

```tex*
EINVAL*```

These scenarios are validated*during kernel startup.

---

## 8.*Relationship to free

Memory obtai*ed from:

```text
alloc()
``*

should be released*using:

```text
free()
```

with*the matching pointer and size.

Se**

    free.md

---

## 9.*Current Limitations

Current limit*tions:

*   No per-process heap
    No user*space heap manager
    Shared kern*l allocator backend
    No allocat*on statistics*exposed to user mode

*hese restrictions are acceptable*for the current development stage.*
---

## 10. Future Enhancements

*lanned improvements:

    Per-proc*ss heaps*    Allocation statistics
    Guar*-page support
    Advanced allocat*on policies
    User-space alloc*tors

---

## Contributor Notes

-*Preserve current errno behavior.
-*Allocation*failures must not panic.
* Keep alloc/free semantics*synchronized.
- Update syscall*harness tests when allocation beha*ior changes.

---

## License

MIT*or Apache 2*0 — TBD.