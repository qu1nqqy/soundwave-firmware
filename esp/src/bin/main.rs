//! soundwave — этап 1: тест ленты.
//!
//! Бегущая радуга по всей ленте + раз в секунду лог в консоль, что живы.
//! Цель — убедиться, что тулчейн, проводка и число пикселей правильные.

// no_std: нет стандартной библиотеки — нет ОС, файлов, потоков, только `core`.
// no_main: нет обычного `fn main()`, точку входа задаёт макрос `#[esp_rtos::main]`.
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Ticker};
use esp_backtrace as _; // при панике печатает backtrace в консоль
use esp_hal::Async;
use esp_hal::clock::CpuClock;
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal_smartled::{RmtSmartLeds, WS2812B_TIMING, buffer_size, color_order};
use esp_radio::wifi::{AuthenticationMethodConfig, sta::StationConfig};
use log::info;
use smart_leds::{RGB8, SmartLedsWriteAsync, brightness, gamma};
use soundwave_core::effects::rainbow;
use soundwave_core::power::power_scale;
use static_cell::StaticCell;

mod wifi;

esp_bootloader_esp_idf::esp_app_desc!();

// --- Настройки ---------------------------------------------------------------

/// 60 LED/м × 3 м
const NUM_LEDS: usize = 180;
/// Глобальная яркость, 0..=255.
const BRIGHTNESS: u8 = 64;
/// Лимит тока на всю ленту. Пока лента питается от USB — держим низко.
/// С отдельным БП 5 В 10 А можно поднять до ~8000.
const MAX_MILLIAMPS: u32 = 500;
/// Частота кадров анимации.
const FPS: u64 = 60;

// --- Типы --------------------------------------------------------------------

/// Частота тактирования RMT — блока, который аппаратно генерирует сигнал для ленты.
const RMT_FREQ: Rate = Rate::from_mhz(80);

/// Драйвер ленты. Внутри — буфер импульсов: 24 импульса (бита) на каждый пиксель.
/// WS2812B ждёт байты в порядке G, R, B — драйвер переставит сам.
type Strip =
    RmtSmartLeds<'static, { buffer_size::<RGB8>(NUM_LEDS) }, Async, RGB8, color_order::Grb>;

/// Буфер драйвера весит ~29 КБ — на стек такое класть нельзя, поэтому
/// он живёт в статической памяти и создаётся один раз при старте.
static STRIP: StaticCell<Strip> = StaticCell::new();

// --- Точка входа -------------------------------------------------------------

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    // Запускаем планировщик embassy на аппаратном таймере.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let mut controller = esp_radio::wifi::WifiController::new(peripherals.WIFI, Default::default())
        .expect("wifi init");
    let config = esp_radio::wifi::Config::Station(
        StationConfig::default()
            .with_ssid(env!("WIFI_SSID").try_into().expect("ssid"))
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                env!("WIFI_PASSWORD").try_into().expect("password"),
            )),
    );
    controller.set_config(&config).expect("wifi config");

    spawner.spawn(wifi::wifi_task(controller).unwrap());

    let rmt = Rmt::new(peripherals.RMT, RMT_FREQ)
        .expect("RMT init")
        .into_async();
    let strip = STRIP.init(
        // memsize = 2: даём каналу 2 блока памяти RMT, чтобы ему было проще
        // успевать подкачивать данные для длинной ленты.
        Strip::new_with_memsize(
            WS2812B_TIMING,
            rmt.channel0,
            peripherals.GPIO18,
            2,
            RMT_FREQ,
        )
        .expect("strip init"),
    );

    info!("soundwave: strip test, {NUM_LEDS} leds");

    let mut frame = [RGB8::default(); NUM_LEDS];
    // Ticker будит нас ровно каждые 1/FPS секунды, даже если кадр считался долго.
    let mut ticker = Ticker::every(Duration::from_hz(FPS));
    let mut frames = 0u32;
    let mut last_report = Instant::now();

    loop {
        // Анимация привязана ко времени, а не к номеру кадра: скорость одинакова при любом FPS.
        // `as u8` специально обрезает число — оттенок «заворачивается» по кругу 0..=255.
        let hue_shift = (Instant::now().as_millis() / 10) as u8;
        rainbow(&mut frame, hue_shift);

        // gamma: глаз воспринимает яркость нелинейно — корректируем, чтобы переходы были ровнее.
        let scale = power_scale(&frame, BRIGHTNESS, MAX_MILLIAMPS);
        strip
            .write(brightness(gamma(frame.iter().copied()), scale))
            .await
            .expect("strip write");

        frames += 1;
        if last_report.elapsed() >= Duration::from_secs(1) {
            info!("alive, fps={frames}, brightness={scale}");
            frames = 0;
            last_report = Instant::now();
        }

        ticker.next().await;
    }
}
