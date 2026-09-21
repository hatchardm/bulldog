# Bulldog Kernel – `close` Syscall

Closes a file descriptor previously returned by `open`.

---

## 1. Prototype

```
close(fd: u64) -> SysResult<()>
```

---

## 2. Behavior

- Releases the file descriptor  
- Decrements reference count on the underlying VFS node  
- Removes the FD from the process’s file descriptor table  
- Returns success on valid FD  

Closing an FD does **not** necessarily free the underlying node immediately —  
future Bulldog versions may support shared descriptors, pipes, or reference‑counted objects.

---

## 3. Errors

- Invalid FD  
- FD not open  
- Underlying VFS node error  

Errors are returned as negative values.

---

## License
MIT or Apache 2.0 — TBD.
