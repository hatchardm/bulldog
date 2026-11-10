## Bulldog Kernel – read Syscall

Reviewed: November 2026

This document describes Bulldog's current read syscall implementation.

The read syscall transfers data from an open file descriptor into user memory.

Related files:

    kernel/src/syscall/read.rs
    write.md
    open.md
    close.md
    syscall.md

---

## 1. Purpose

The read syscall reads data from an open file descriptor.

Current syscall number:

    4

Supported descriptor types currently include:

    VFS-backed files

---

## 2. Prototype

```text
read(fd: u64, buf: *mut u8, len: usize) -> ssize_t
```*
Arguments:

    fd
        Source*file descriptor

    buf
        D*stination user buffer

    len*        Maximum number of bytes re*uested

Return value:

    Number *f bytes read

or

    Negative err*o value on failure

---

##*3. Implementation

Current impleme*tation:

```rust
sys_read(fd, buf_*tr, len)
```

Operation flow:

   *Validate FD
            ↓
    Read*into kernel buffer
            ↓*    Copy kernel buffer to user mem*ry
            ↓
    Return byte c*unt

The syscall never reads direc*ly into user memory.

---

##*4. Successful Reads

Successful re*ds return:

```text
bytes_read
```*
Example:

```text
read(fd, buffer* 16)
```

Result:

```text
8
```

*he syscall harness validates succe*sful reads from*

```text
/etc/hostname*```

---

## 5. Zero-Length*Reads

Current behavior:

```text
*en ==*0
```

returns:

```text
*
```

without accessing the file d*scriptor.

Zero-length reads are c*nsidered successful.

---

## 6. B*ffer Handling

The syscall first r*ads into a temporary kernel buffer*

Current implementation uses:

``*text
MAX_READ = 4096
```

Maximum *ytes processed per syscall:

```te*t
4096
```

Requests larger than*this are currently clamped.

---

*# 7. Error Handling

### EBADF

Re*urned when:

    File descriptor i* invalid
    File descriptor has b*en closed

Examples:

```text
fd*= 99
fd = closed*fd
```

---

### EFAULT

Returned*when:

    User buffer cannot be w*itten*
Example:

```text
*uf = 0xffffffffffffffff
```

*--

## 8. Harness Coverage

The sy*call harness validates:

### Succe*sful read

```text
read(valid_fd, *alid_buffer, len)
``*

Expected:

```text
bytes read
``*

### Invalid file descriptor

```*ext
read(99, ...)
```

Expected:

*``text*EBADF
```

### Read after*close

```text
read(closed_fd,*...)
```

Expected:

```text
EBADF*```

### Invalid user*buffer

```text
read(fd, bogus_ptr* len)
``*

Expected:

```text
*FAULT
```

### Zero length

```*ext
read(fd, buf, 0)
*``

Expected:

```text
0
```

*--

## 9. Current Limitations

*urrent implementation:

    Uses t*mporary fixed-size buffer
*   Limits reads to 4096 bytes
    *oes not support vectored I/O
    D*es not support async I/O

These li*itations are acceptable*for the current development stage.*
---

## 10. Future Enhancements

*lanned improvements:

   *Dynamic read buffering
    Larger *ransfers
    Scatter/gather I/O
  * Async I/O
    Permission checks
 *  Expanded VFS integration

---

#* Contributor Notes*
- Preserve current errno behavior*
- Preserve zero-length read seman*ics.
- Keep MAX_READ synchronized *ith implementation.
- Update sysca*l harness coverage whenever behavi*r changes.

---

## License

MIT o* Apache 2.0 — TBD.