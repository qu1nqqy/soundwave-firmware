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

pub static COMMANDS: Signal<CriticalSectionRawMutex, Command> = Signal::new();

#[embassy_executor::task]
pub async fn udp_task(netstack: Stack<'static>) {
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

    socket
        .bind(soundwave_core::protocol::PORT)
        .expect("udp bind");

    let mut buf = [0u8; 64];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((n, from)) => match parse(&buf[..n]) {
                Ok(cmd) => COMMANDS.signal(cmd),
                Err(e) => warn!("bad packet from: {}: {:?}", from, e),
            },
            Err(e) => {
                info!("{:?}", e);
            }
        }
    }
}
