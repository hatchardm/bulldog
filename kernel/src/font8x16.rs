pub static FONT_RAW: &[u8] = include_bytes!("../font8x16.bin");

pub fn font8x16() -> &'static [u8] {
    // glyphs are the first 4096 bytes, just like the bootloader uses
    &FONT_RAW[..4096]
}

