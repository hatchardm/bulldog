# Bulldog Kernel – Virtual File System (VFS) Interface

This document describes the user‑visible VFS interface exposed by Bulldog’s syscall layer.

It complements:

- `open.md`
- `read.md`
- `write.md`
- `execve.md`
- `process.md`

---

## 1. Overview

Bulldog’s VFS provides:

- Path resolution  
- File descriptor allocation  
- Read/write operations  
- Integration with `execve`  
- Kernel‑internal node abstraction  

The VFS is intentionally simple at this stage.

---

## 2. Path Resolution

Paths are resolved using:

- Root directory `/`
- Relative paths (future)
- Mount table (future)

Resolution returns a VFS node.

---

## 3. File Descriptors

Opening a file returns an FD:

```
fd = open(path)
```

FDs map to VFS nodes.

FDs are:

- Process‑local  
- Inherited across `fork()`  
- Replaced across `execve()`  

---

## 4. Read/Write

### Read

```
read(fd, buf, len)
```

Reads up to `len` bytes into `buf`.

### Write

```
write(fd, buf, len)
```

Writes `len` bytes from `buf`.

---

## 5. Exec Integration

`execve(path, argv, envp)` uses VFS to:

- Open the ELF file  
- Read its contents  
- Pass the node to the ELF loader  

---

## 6. Future Enhancements

- Directories  
- Mount table  
- Block devices  
- Pipes  
- Sockets  
- mmap  
- Permissions  

---

## License
MIT or Apache 2.0 — TBD.
