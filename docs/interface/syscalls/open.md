# Bulldog Kernel – `open` Syscall

Opens a file via the VFS.

---

## 1. Prototype

```
open(path: *const u8) -> SysResult<u64>
```

Returns file descriptor.

---

## 2. Behavior

- Resolves path  
- Allocates FD  
- Returns FD  

---

## 3. Errors

- File not found  
- Invalid path  

---

## License
MIT or Apache 2.0 — TBD.
