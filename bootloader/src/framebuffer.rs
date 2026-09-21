use crate::color::Color;

pub struct Framebuffer<'a> {
    pub data: &'a mut [u8],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
}

impl<'a> Framebuffer<'a> {
    pub fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = (y * self.stride + x) * 4;
        self.data[idx + 0] = color.b;
        self.data[idx + 1] = color.g;
        self.data[idx + 2] = color.r;
        self.data[idx + 3] = 0x00;
    }

    pub fn fill_color(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        let max_x = (x + w).min(self.width);
        let max_y = (y + h).min(self.height);

        for yy in y..max_y {
            for xx in x..max_x {
                self.put_pixel(xx, yy, color);
            }
        }
    }

    pub fn clear(&mut self) {
        self.fill_color(Color::BLACK);
    }

    pub fn draw_line(&mut self, mut x0: isize, mut y0: isize, x1: isize, y1: isize, color: Color) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 {
            self.put_pixel(x0 as usize, y0 as usize, color);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

}
