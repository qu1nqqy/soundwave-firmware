use super::*;

const WHITE: RGB8 = RGB8::new(255, 255, 255);
const RED: RGB8 = RGB8::new(255, 0, 0);

#[test]
fn white_pixel_is_60_ma() {
    assert_eq!(
        estimate_ma(&[WHITE], 255),
        3 * MA_PER_CHANNEL + MA_IDLE_PER_LED
    );
}

#[test]
fn dark_frame_draws_only_idle_current() {
    let frame = [RGB8::default(); 180];
    assert_eq!(estimate_ma(&frame, 255), 180 * MA_IDLE_PER_LED);
}

#[test]
fn brightness_scales_current() {
    let frame = [RED; 100];
    let full = estimate_ma(&frame, 255);
    let half = estimate_ma(&frame, 128);
    assert!(half < full);
    assert_eq!(estimate_ma(&frame, 0), 100 * MA_IDLE_PER_LED);
}

#[test]
fn keeps_brightness_under_limit() {
    let frame = [RED; 10]; // ~210 мА на полной яркости
    assert_eq!(power_scale(&frame, 255, 500), 255);
}

#[test]
fn lowers_brightness_over_limit() {
    // 180 белых пикселей — ~10,8 А, а у нас 500 мА.
    let frame = [WHITE; 180];
    let scale = power_scale(&frame, 255, 500);
    assert!(scale < 255);
    assert!(estimate_ma(&frame, scale) <= 500);
    // И это максимально возможная яркость: на единицу больше — уже перебор.
    assert!(estimate_ma(&frame, scale + 1) > 500);
}

#[test]
fn never_exceeds_wanted() {
    let frame = [RED; 1];
    assert_eq!(power_scale(&frame, 64, 10_000), 64);
}

#[test]
fn limit_below_idle_turns_off() {
    let frame = [WHITE; 180];
    assert_eq!(power_scale(&frame, 255, 100), 0);
}

#[test]
fn empty_frame() {
    assert_eq!(estimate_ma(&[], 255), 0);
    assert_eq!(power_scale(&[], 255, 0), 255);
}
