//! Задача Wi-Fi: держит плату подключённой к роутеру и переподключает после разрыва.

use embassy_time::{Duration, Timer};
use esp_radio::wifi::WifiController;
use log::info;

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
                info!("task: wifi: connect error: {:?}", e);
                // Нет сети или неверный пароль — пауза, чтобы не долбить роутер, и снова.
                Timer::after(Duration::from_secs(5)).await;
            }
        }
    }
}
