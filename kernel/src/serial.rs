// kernel/src/serial.rs
use x86_64::instructions::port::Port;

/// Write a single byte to COM1 serial port (0x3F8).
pub fn serial_write_byte(byte: u8) {
    unsafe {
        let mut port = Port::new(0x3F8);
        port.write(byte);
    }
}

/// Print a string to COM1 serial port.
pub fn serial_print(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

/// Print a string followed by '\n'.
pub fn serial_println(s: &str) {
    serial_print(s);
    serial_write_byte(b'\n');
}

/// Print a u64 as decimal (no alloc, no formatting macros).
pub fn serial_print_u64(mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;

    if n == 0 {
        serial_write_byte(b'0');
        return;
    }

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        serial_write_byte(buf[i]);
    }
}

/// Print a u64 as 0xXXXXXXXXXXXXXXX hex.
pub fn serial_print_hex_u64(v: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    serial_print("0x");

    for shift in (0..64).rev().step_by(4) {
        let nibble = ((v >> shift) & 0xF) as usize;
        serial_write_byte(HEX[nibble]);
    }
}



