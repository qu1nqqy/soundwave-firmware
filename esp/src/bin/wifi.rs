use embassy_time::{Duration, Timer};
use esp_radio::wifi::WifiController;
use log::info;

#[allow(
    clippy::large_stack_frames,
    reason = "состояние embassy-задачи живёт в статической памяти пула задач, а не на стеке"
)]
#[embassy_executor::task]
pub async fn wifi_task(mut controller: WifiController<'static>) {
    loop {
        match controller.connect_async().await {
            Ok(info) => {
                info!("task: wifi: connected, {:?}", info);
                let _ = controller.wait_for_disconnect_async().await;
            }
            Err(e) => {
                info!("task: wifi: connect error: {:?}", e);
                Timer::after(Duration::from_secs(5)).await;
            }
        }
    }
}
