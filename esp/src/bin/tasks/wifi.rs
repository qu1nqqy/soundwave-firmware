//! Задача Wi-Fi: держит плату подключённой к роутеру и переподключает после разрыва.

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
use esp_radio::wifi::{AuthenticationMethodConfig, Config, WifiController, sta::StationConfig};
use log::{info, warn};

/// Создаёт контроллер Wi-Fi, задаёт сеть и запускает `wifi_task`.
///
/// Контроллер — это «радио»: найти сеть и подключиться по SSID и паролю.
/// Имя и пароль подставляются при сборке (`env!`) из esp/.cargo/wifi.toml — в git их нет.
#[allow(
    clippy::large_stack_frames,
    reason = "вызывается один раз при старте; конфиг и контроллер крупные, но живут лишь мгновение"
)]
pub fn start(spawner: Spawner, wifi: WIFI<'static>) {
    let mut controller = WifiController::new(wifi, Default::default()).expect("wifi init");
    let config = Config::Station(
        StationConfig::default()
            .with_ssid(env!("WIFI_SSID").try_into().expect("ssid"))
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                env!("WIFI_PASSWORD").try_into().expect("password"),
            )),
    );
    controller.set_config(&config).expect("wifi config");
    spawner.spawn(wifi_task(controller).unwrap());
}

#[allow(
    clippy::large_stack_frames,
    reason = "состояние embassy-задачи живёт в статической памяти пула задач, а не на стеке"
)]
#[embassy_executor::task]
pub async fn wifi_task(mut controller: WifiController<'static>) {
    // Вечный цикл: подключились → ждём разрыва → подключаемся снова.
    loop {
        // SSID и пароль уже заданы в main через set_config — тут только «постучаться» к роутеру.
        match controller.connect_async().await {
            Ok(info) => {
                info!("task: wifi: connected, {:?}", info);
                // «Висит», пока связь есть, и возвращается, когда роутер отвалился.
                // Причина разрыва нам не нужна — сразу идём на новый круг.
                let _ = controller.wait_for_disconnect_async().await;
            }
            Err(e) => {
                warn!("task: wifi: connect error: {:?}", e);
                // Нет сети или неверный пароль — пауза, чтобы не долбить роутер, и снова.
                Timer::after(Duration::from_secs(5)).await;
            }
        }
    }
}
