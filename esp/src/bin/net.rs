use embassy_net::Runner;
use esp_radio::wifi::Interface;

#[allow(
    clippy::large_stack_frames,
    reason = "состояние embassy-задачи живёт в статической памяти пула задач, а не на стеке"
)]
#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await;
}
