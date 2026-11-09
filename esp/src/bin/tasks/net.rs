//! Задача сетевого стека — «мотор» embassy-net. Без неё не работают ни DHCP, ни сокеты.

use embassy_net::Runner;
use esp_radio::wifi::Interface;

#[allow(
    clippy::large_stack_frames,
    reason = "состояние embassy-задачи живёт в статической памяти пула задач, а не на стеке"
)]
#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, Interface>) {
    // Вечно крутит стек: гоняет пакеты через Wi-Fi-интерфейс, обслуживает DHCP и сокеты.
    // Никогда не возвращается.
    runner.run().await;
}
