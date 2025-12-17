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
