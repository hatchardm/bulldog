fn main() {
    // Write to stdout (fd = 1)
    let _ = bulldog_syscall::write(1, b"Hello from userspace!\n");

    // Exit cleanly
    bulldog_syscall::exit(0);
}