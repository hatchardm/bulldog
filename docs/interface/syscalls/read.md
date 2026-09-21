# Bulldog Kernel – `read` Syscall

Reads data from a file descriptor.

---

## 1. Prototype

```
read(fd: u64, buf: *mut u8, len: usize) -> SysResult<usize>
```

---

## 2. Behavior

- Reads up to `len` bytes  
- Writes into `buf`  
- Returns number of bytes read  

---

## 3. Errors

- Invalid FD  
- Read error  

---

## License
MIT or Apache 2.0 — TBD.
