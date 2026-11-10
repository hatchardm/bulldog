## Bulldog Kernel – free Syscall

Reviewed: November 2026

This document describes Bulldog's current memory deallocation syscall.

The free syscall releases memory previously obtained through:

    alloc()

Related files:

    kernel/src/syscall/free.rs
    alloc.md
    syscall.md
    syscall-table.md

---

## 1. Purpose

The free syscall releases dynamically allocated memory back to the kernel allocator.

Current syscall number:

    6

Memory passed to free() should originate from a successful:

    alloc()

call.

---

## 2. Prototype

```text
free(ptr: *mut u8, size: usize) -> int
```
*Arguments:

    ptr
        Pointe* previously returned by alloc()

 *  size
        Allocation size ori*inally requested

Return value:

 *  0 on success

    Negative errno*value on failure

---

## 3. Imple*entation

Current implementation:
*```rust
sys_free(ptr, size)
```

M*mory is released using:

```rust
a*loc::alloc::dealloc()
```

with:

*``rust
Layout::from_size_align(siz*, 8)
``*

Current alignment:

    8 bytes
*---

*# 4. Successful Deallocation

On*success:

```text
free(ptr, size)
*``

returns:

```text
0
```

The m*mory region is returned to the all*cator.

---

## 5. Validation Rule*

### Pointer Must Be Non-Zero*
Rejected:

```text
ptr == 0*```

Result:

```text
EINVAL
*``

---

### Size Must Be Non-Zero*
Rejected:

```text
size == 0*```

Result:

```text
EINVAL
*``

---

### Pointer Must Fall*Within Heap

Current*implementation verifies the pointe* lies within:

*``text
HEAP_START*HEAP_START + HE*P_SIZE
```

Pointers*outside the heap are rejected.

Re*ult:

```*ext
EINVAL
```

---

### Layout Mu*t Be Valid

If a*valid allocation layout cannot be *reated:

```text
EINVAL
``*

is returned.

---

## 6. Error*Handling

### EINVAL

Returned whe**

    Pointer is zero
    Size is *ero
    Pointer falls outside the*heap
    Layout construction fails*
Errors are returned*through the syscall errno mechanis*.

---

## 7. Harness Coverage

Th* syscall harness validates:

### S*ccessful free

```text
alloc(64)
f*ee*ptr, 64)
``*

Expected:

```text
0
```

*## Null pointer

```text
*ree(0, 64)
```

Expected*

```text
EINVAL
```

### Zero*size

```text
free(ptr, 0*
```

Expected:

```text
EINVAL
*``

### Invalid pointer

```text
*ree(0xffffffffffffffff, 64)
*``

Expected:

```text
EINVAL
``*

---

## 8. Relationship To*alloc

Memory should be allocated*using:

```text
alloc()
```

and*released using:

```text
free*)
```

with matching pointer and s*ze arguments*

See:

    alloc.md

---

## *. Current Limitations

Current imp*ementation*does not track*

    Allocation ownership
    Dou*le-free detection
    Allocation m*tadata validation
    Per-process *eaps*
Callers must provide valid argume*ts.

---

## 10. Future Enhancemen*s

Planned improvements:

    Doub*e-free detection
   *Allocation tracking
    Per-proces* heaps
    Heap statistics
    Deb*g allocation checking

---

## Con*ributor Notes

- Preserve current *rrno behavior.
- Keep alloc()*and free() semantics synchronized.*- Validation failures*must not panic.
- Update syscall h*rness tests whenever behavior chan*es.

---

## License

MIT or Apach* 2.0 — TBD.