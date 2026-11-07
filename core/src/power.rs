//! Оценка тока ленты и ограничение яркости, чтобы не перегрузить блок питания.

use smart_leds::RGB8;

/// Каждый канал пикселя на полной яркости ест ~20 мА.
pub const MA_PER_CHANNEL: u32 = 20;
/// Сам чип пикселя ест ~1 мА, даже когда не светится.
pub const MA_IDLE_PER_LED: u32 = 1;

/// Примерный ток кадра в миллиамперах при глобальной яркости `brightness` (0..=255).
pub fn estimate_ma(frame: &[RGB8], brightness: u8) -> u32 {
    let idle_ma = MA_IDLE_PER_LED * frame.len() as u32;
    full_ma(frame) * brightness as u32 / 255 + idle_ma
}

/// Возвращает `wanted`, если кадр при такой яркости укладывается в `max_milliamps`,
/// иначе — наибольшую яркость, которая в лимит укладывается.
pub fn power_scale(frame: &[RGB8], wanted: u8, max_milliamps: u32) -> u8 {
    if estimate_ma(frame, wanted) <= max_milliamps {
        return wanted;
    }
    let idle_ma = MA_IDLE_PER_LED * frame.len() as u32;
    let budget = max_milliamps.saturating_sub(idle_ma);
    (budget * 255 / full_ma(frame).max(1)).min(wanted as u32) as u8
}

/// Ток всех каналов на полной яркости, без учёта холостого тока чипов.
fn full_ma(frame: &[RGB8]) -> u32 {
    // Сумма всех каналов всех пикселей: 255 на канале = MA_PER_CHANNEL.
    let sum: u32 = frame
        .iter()
        .map(|px| px.r as u32 + px.g as u32 + px.b as u32)
        .sum();
    sum * MA_PER_CHANNEL / 255
}

#[cfg(test)]
mod tests {
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
}
