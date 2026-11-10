## Bulldog Kernel – write Syscall

Reviewed: November 2026

This document describes Bulldog's current write syscall implementation.

The write syscall transfers data from user memory to an open file descriptor.

Related files:

    kernel/src/syscall/write.rs
    read.md
    open.md
    close.md
    syscall.md

---

## 1. Purpose

The write syscall writes data to an open file descriptor.

Current syscall number:

    1

Supported descriptor types currently include:

    Standard output
    VFS-backed files

---

## 2. Prototype

```text
write(fd: u64, buf: *const u8,*len: usize) -> ssize*t
```

Arguments:

    fd
       *Destination file descriptor

    b*f
        Pointer to user buffer

*   len
        Number of bytes req*ested

Return value:

   *Number of bytes written

or*
    Negative errno value on failu*e

---

## 3. Implementation

Curr*nt implementation:

```rust
sys_wr*te(fd, buf_ptr, len*
```

Operation flow:

    Validat* FD
            ↓
    Copy user bu*fer
            ↓
    Write throug* file interface
            ↓
    *eturn byte count

The syscall copi*s data from user*space before*performing the write.

---

## 4* Successful Writes

Successful*writes return:

```text
bytes*written
```

Example:

*``text
write(fd, buffer* 20)
```

Result*

```text
20
```

The syscall*harness validates successful write*.

---

## 5.*Zero-Length Writes

Current behavi*r:

```text
len == 0
``*

returns:

```text
0
```

*ithout accessing the file descript*r.

Zero-length writes are conside*ed successful.

---

## 6.*Buffer Handling

The syscall copie* user buffers into a temporary ker*el buffer before writing.

Current*implementation uses*

```text
MAX_WRITE = 4096
```

Ma*imum*bytes processed*per syscall:

```text
409*
```

Requests larger than this ar* currently clamped.

---

*# 7. Error Handling

### EB*DF

Returned when:

    File descr*ptor is invalid*    File descriptor is unsupported*
Examples:

*``text
fd = *9
fd = 0
```

---

### EFAULT

Ret*rned when:

    User buffer cannot*be copied

Example:

```text
buf =*0xffffffffffffffff
```

---

*# 8. Harness Coverage*
The syscall harness validates:

#*# Successful*write

```text
write*valid_fd, valid_buffer,*len)
```

Expected:

```text
bytes*written
```

### Invalid*file descriptor

```text
*rite(99, ...)
```

Expected:

```*ext
EBADF
```

### stdin*descriptor

```text
write*0, ...)
```

Expected:

```text*EBADF
```

### Invalid pointer*
```text
write(fd* bogus_ptr, len)
```

Expected:

*``text
EFAULT
```

### Zero*length

```text
write(fd* buf, 0)
```

Expected*

```text
0
```

*## Large write

```text
write(fd, *uf, very_large_len)
```

Expected:*
```text
clamped to MAX_WRITE*```

---

## 9. Current Limit*tions

Current implementation:

  * Uses temporary fixed-size buffer
*   Limits writes to 4096 bytes
   *Does not support vectored I*O
    Does not support async I*O

These limitations are acceptabl* for the current*development stage.

---

## 10. Fu*ure Enhancements

Planned improvem*nts:

    Dynamic write buffering
*   Larger transfers
    Scatter/g*ther I/O
    Async I*O
    Permission checking*    Improved VFS integration

---
*## Contributor Notes

- Preserve c*rrent errno behavior*
- Preserve zero-length write*semantics.
-*Keep MAX_WRITE synchronized with i*plementation.
-*Update harness coverage whenever w*ite behavior changes.

---

## Lic*nse

MIT or Apache 2.0 — TBD.
````*