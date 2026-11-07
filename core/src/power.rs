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

