use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libconcentratord::jitqueue::TxPacket;
use libconcentratord::signals::Signal;
use libconcentratord::{jitqueue, stats};
use libloragw_2g4::hal;

use super::super::wrapper;

pub fn jit_loop(
    queue: Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
    antenna_gain: i8,
    stop_receive: Receiver<Signal>,
) {
    debug!("Start JIT queue loop");

    loop {
        // Instead of a 10ms sleep, we receive from the stop channel with a
        // timeout of 10ms.
        match stop_receive.recv_timeout(Duration::from_millis(10)) {
            Ok(v) => {
                debug!("Received stop signal, signal: {}", v);
                break;
            }
            _ => {}
        };

        let tx_packet = match get_tx_packet(&queue) {
            Some(v) => v,
            None => continue,
        };

        let downlink_id = tx_packet.get_id();
        let mut tx_packet = tx_packet.tx_packet();
        tx_packet.rf_power = tx_packet.rf_power - antenna_gain;

        match hal::send(&tx_packet) {
            Ok(_) => {
                info!("Scheduled packet for TX, downlink_id: {}, count_us: {}, freq: {}, bw: {}, mod: {:?}, dr: {:?}",
                    downlink_id,
                    tx_packet.count_us,
                    tx_packet.freq_hz,
                    tx_packet.bandwidth,
                    hal::Modulation::LoRa,
                    tx_packet.datarate
                    );

                stats::inc_tx_packets_emitted();
            }
            Err(err) => {
                error!("Schedule packet for tx error, error: {}", err);
            }
        }
    }
}

fn get_tx_packet(
    queue: &Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
) -> Option<wrapper::TxPacket> {
    let concentrator_count = hal::get_instcnt().expect("get concentrator count error");
    let mut queue = queue.lock().unwrap();
    return queue.pop(concentrator_count);
}
