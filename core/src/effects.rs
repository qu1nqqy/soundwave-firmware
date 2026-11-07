//! Эффекты: функции, которые заполняют кадр цветами.

use smart_leds::RGB8;
use smart_leds::hsv::{Hsv, hsv2rgb};

/// Растягивает полный круг цветов (256 оттенков) на всю ленту и сдвигает его на `shift`.
pub fn rainbow(frame: &mut [RGB8], shift: u8) {
    let len = frame.len();
    for (i, px) in frame.iter_mut().enumerate() {
        let hue = (i * 256 / len) as u8;
        *px = hsv2rgb(Hsv {
            hue: hue.wrapping_add(shift),
            sat: 255,
            val: 255,
        });
    }
}

#[cfg(test)]
mod tests;
