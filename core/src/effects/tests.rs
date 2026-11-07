use super::*;

fn hue(h: u8) -> RGB8 {
    hsv2rgb(Hsv {
        hue: h,
        sat: 255,
        val: 255,
    })
}

#[test]
fn empty_frame_does_nothing() {
    rainbow(&mut [], 42);
}

#[test]
fn first_pixel_has_shift_hue() {
    let mut frame = [RGB8::default(); 10];
    rainbow(&mut frame, 100);
    assert_eq!(frame[0], hue(100));
}

#[test]
fn covers_whole_hue_circle() {
    // 256 пикселей — ровно по одному на каждый оттенок.
    let mut frame = [RGB8::default(); 256];
    rainbow(&mut frame, 0);
    for (i, px) in frame.iter().enumerate() {
        assert_eq!(*px, hue(i as u8), "pixel {i}");
    }
}

#[test]
fn shift_moves_rainbow_by_one_pixel() {
    let mut base = [RGB8::default(); 256];
    let mut shifted = [RGB8::default(); 256];
    rainbow(&mut base, 0);
    rainbow(&mut shifted, 1);
    for i in 0..255 {
        assert_eq!(shifted[i], base[i + 1], "pixel {i}");
    }
    // Последний пиксель «заворачивается» в начало круга.
    assert_eq!(shifted[255], base[0]);
}

#[test]
fn hue_spreads_over_short_strip() {
    // На 4 пикселях оттенки идут с шагом в четверть круга.
    let mut frame = [RGB8::default(); 4];
    rainbow(&mut frame, 0);
    assert_eq!(frame, [hue(0), hue(64), hue(128), hue(192)]);
}
