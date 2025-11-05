#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        if let Some(writer) = $crate::writer::WRITER.lock().as_mut() {
            let _ = writer.write_fmt(format_args!($($arg)*));
        }
    });
}


#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
