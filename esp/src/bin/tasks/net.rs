//! Задача сетевого стека — «мотор» embassy-net. Без неё не работают ни DHCP, ни сокеты.

use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use esp_radio::wifi::Interface;
use log::info;
use static_cell::StaticCell;

/// Память под внутренние структуры сетевого стека: по слоту на каждый сокет
/// (DHCP, наш UDP и один про запас). Стек живёт до выключения платы,
/// поэтому и она — в статической памяти, создаётся один раз при старте.
static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

/// Поднимает сетевой стек, запускает `net_task` и ждёт, пока роутер выдаст IP по DHCP.
#[allow(
    clippy::large_stack_frames,
    reason = "вызывается один раз при старте; стек и runner крупные, но runner сразу уезжает в задачу"
)]
pub async fn start(spawner: Spawner) -> Stack<'static> {
    // Интерфейс — «сетевая карта», через которую идут сами пакеты;
    // station — плата клиент роутера (а не своя точка доступа). Создаётся ровно один раз.
    let interface = Interface::station();
    let config = embassy_net::Config::dhcpv4(Default::default());
    let resources = RESOURCES.init(StackResources::new());
    // Случайное число для сетевых номеров (порты, ID DHCP-запросов). Rng даёт u32 — склеиваем
    // два в u64. Сначала расширяем до u64, потом сдвигаем: сдвиг u32 на 32 потерял бы все биты.
    let rng = esp_hal::rng::Rng::new();
    let seed = u64::from(rng.random()) << 32 | u64::from(rng.random());
    let (stack, runner) = embassy_net::new(interface, config, resources, seed);

    // runner запускаем ДО ожидания IP: DHCP делает именно он. В обратном порядке ждали бы вечно.
    spawner.spawn(net_task(runner).unwrap());
    stack.wait_config_up().await;
    info!("ip: {:?}", stack.config_v4());
    stack
}

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
