## Bulldog Kernel – close Syscall

Reviewed: November 2026

Current syscall number:

    7

### Prototype

close(fd: u64) -> int

Returns:

    0 on success
    negative errno on failure

### Behavior

- Releases the file descriptor
- Removes the descriptor from the FD table
- Makes the descriptor available for reuse
- Returns 0 on success

Closed descriptors may later be reallocated by open().

### Error Handling

EBADF

Returned when:

    FD is invalid
    FD is not currently open

### Harness Coverage

Validated paths:

    Valid close
    Invalid FD
    FD reuse after close

### License

MIT or Apache 2.0 — TBD.
