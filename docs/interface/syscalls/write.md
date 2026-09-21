# Bulldog Kernel – `write` Syscall

Writes data to a file descriptor.

---

## 1. Prototype

```
write(fd: u64, buf: *const u8, len: usize) -> SysResult<usize>
```

---

## 2. Behavior

- Writes `len` bytes  
- Returns number of bytes written  

---

## 3. Errors

- Invalid FD  
- Write error  

---

## License
MIT or Apache 2.0 — TBD.
