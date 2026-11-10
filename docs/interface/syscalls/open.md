## Bulldog Kernel – open Syscall

Reviewed: November 2026

This document describes Bulldog's current open syscall implementation.

The open syscall resolves a path, creates a file-descriptor entry, and returns a handle suitable for use with:

    read()
    write()
    close()

Related files:

    kernel/src/syscall/open.rs
    read.md
    write.md
    close.md
    syscall.md

---

## 1. Purpose

The open syscall opens a file and returns a file descriptor.

Current syscall number:

    3

The syscall currently supports:

    VFS-backed files
    Kernel-managed file descriptors

---

## 2. Prototype

```text
open(path: *const u8, flags: u64, mode: u*4) -> int
```

Arguments:

*   path
        Null-terminated pa*h string

    flags*        Open flags*
    mode
        Reserved*for future use

Return value*

    File descriptor on success

*r*
    Negative errno value on failu*e

---

## 3. Implementation

Curr*nt implementation:

```rust
sys_op*n(path_ptr, flags, mode)
```

Oper*tion flow:

    Validate path poin*er
            ↓
    Copy path fro* user memory
            ↓
    Val*date path and*flags
            ↓
    Resolve V*S object
            ↓
    Create *ile*descriptor
            ↓
    Retur* FD

---

## 4.*VFS Integration

Paths*beginning with:

```text
/
```

*re treated as VFS paths.

Resoluti*n is performed through:

```rust*resolve_path()
```

The*returned object is wrapped using:
*```rust
V*sFileLike
```

and*assigned a file descriptor.

Examp*es:

```text
/etc/*ostname
/var/log/test_write.txt
``*

---

## 5. Successful Opens*
On success:

*``text
open(...)
*``

returns:

```text
fd*>= 3
```

Examples*

```text
fd = 3
fd*= 4
fd = 5
...
*``

The file descriptor can*then be passed to*

```text
read()
write()
close()
`*`

---

## 6. File Offset Handling*
Current V*S opens perform:

```text*rewind()
```

before the*descriptor is returned.

This ensu*es reopened files begin at*offset zero.

---

## *. Error Handling

### EN*ENT

Returned when:

    Path poin*er is null
    Path*cannot be*resolved

---

### EFAULT

Returne* when:

    User path pointer cann*t be read

Example:

```text
0*ffffffffffffffff
```

---

### EIN*AL

Returned when:

    Path*is empty
    Unsupported flags sup*lied*
Example:

```text
flags*= 0xFFFFFFFF
```

---

###*EMFILE

Returned when:

    No fil* descriptors remain available

The*syscall harness validates this beh*vior.

---

## 8. Harness Coverage*
The syscall harness validates:

#*# Successful open

```text
open("/*tc/hostname")
```

Expected:

```t*xt
Valid FD
```

### Invalid point*r

```text
open(bogus_ptr)
*``

Expected:

```text
EFAULT
*``

### Empty path

```text
open*"")
```

Expected:

```text
EINVAL**``

### Invalid flags

```text
fla*s = 0xFFFFFFFF
```

Expected:

*``text
EINVAL
```

### FD exhausti*n

Repeated opens*until descriptor limit reached.

E*pected:

*``text
EMFILE
```

### FD reuse

C*osed descriptors become reusable.
*Expected*

```text
Lowest available FD reus*d
```

---

## 9. Current Limitati*ns

Current implementation:

    U*es fixed FD table
    Supports a l*mited flag set
    Does not enforc* permissions
    Uses shared V*S objects

These limitations are a*ceptable at the current developmen* stage.

---

## 10. Future Enhanc*ments

Planned improvements:

    *dditional open flags
    Permissio* enforcement
    Access modes
    *er-process FD tables
    Symbolic *inks
    Directory descriptors

--*

## Contributor Notes

- Preserve*current errno*behavior.
- Keep open/read*write/close documentation synchron*zed.
- Maintain FD allocation sema*tics.
- Preserve file rewind behav*or on open.
- Update harness cover*ge whenever open behavior changes.*
---

## License

MIT or Apache 2.* — TBD.