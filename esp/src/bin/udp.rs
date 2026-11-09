//! Задача UDP: принимает пакеты от мака, разбирает их и передаёт команды циклу ленты.

#![allow(
    clippy::large_stack_frames,
    reason = "состояние embassy-задачи (вместе с буферами сокета) живёт в статической памяти пула задач, а не на стеке"
)]

use embassy_net::{
    Stack,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use log::{info, warn};
use soundwave_core::protocol::{Command, parse};

/// Почтовый ящик на одно сообщение: `udp_task` кладёт команду (`signal`), цикл ленты в `main`
/// забирает (`try_take`). Если старую ещё не забрали, новая её заменяет — важен только свежий цвет.
/// `CriticalSectionRawMutex`: на миг «положить/забрать» отключаются прерывания, поэтому ящик
/// безопасен и между задачами, и между двумя ядрами.
pub static COMMANDS: Signal<CriticalSectionRawMutex, Command> = Signal::new();

#[embassy_executor::task]
pub async fn udp_task(netstack: Stack<'static>) {
    // Память для сокета (в no_std он не выделяет её сам): rx — входящие, tx — исходящие.
    // *_meta — опись пакетов (длина, отправитель), *_buffer — сами байты. Отправлять пока нечего,
    // но `UdpSocket::new` требует обе пары.
    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut tx_meta = [PacketMetadata::EMPTY; 8];

    let mut rx_buffer = [0u8; 512];
    let mut tx_buffer = [0u8; 512];

    let mut socket = UdpSocket::new(
        netstack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    // Порт берём из протокола в core — так прошивка и мак не разъедутся.
    socket
        .bind(soundwave_core::protocol::PORT)
        .expect("udp bind");

    // Сюда кладётся один пакет. Самый длинный — палитра из 5 цветов, 24 байта; 64 — с запасом.
    let mut buf = [0u8; 64];

    loop {
        // Ждём пакет; пока ждём, остальные задачи (и лента) работают дальше.
        // n — сколько байт пришло: смотреть надо только на &buf[..n], а не на весь buf.
        match socket.recv_from(&mut buf).await {
            Ok((n, from)) => match parse(&buf[..n]) {
                Ok(cmd) => COMMANDS.signal(cmd),
                Err(e) => warn!("bad packet from: {}: {:?}", from, e),
            },
            // Ошибка самого сокета — например, пакет не влез в buf.
            Err(e) => {
                info!("{:?}", e);
            }
        }
    }
}
