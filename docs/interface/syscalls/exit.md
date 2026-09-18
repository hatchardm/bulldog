# Bulldog Kernel – `exit` Syscall

Terminates the current process.

---

## 1. Prototype

```
exit(code: u64) -> !
```

---

## 2. Behavior

- Marks process as terminated  
- Removes from scheduler  
- Frees resources  
- Never returns  

---

## License
MIT or Apache 2.0 — TBD.
