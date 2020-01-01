use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use jitqueue::TxPacket;
use libloragw_sx1301::hal;

use super::super::wrapper;
use super::stats;
use super::timersync;

pub fn jit_loop(queue: Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>, antenna_gain: i8) {
    debug!("Starting JIT queue loop");

    loop {
        thread::sleep(Duration::from_millis(10));

        let tx_packet = match get_tx_packet(&queue) {
            Some(v) => v,
            None => continue,
        };

        let downlink_id = tx_packet.get_id();
        let mut tx_packet = tx_packet.tx_packet();
        tx_packet.rf_power = tx_packet.rf_power - antenna_gain;

        match hal::send(&tx_packet) {
            Ok(_) => {
                info!(
                    "Scheduled packet for TX, downlink_id: {}, count_us: {}, freq: {}, bw: {}, mod: {:?}, dr: {:?}",
                    downlink_id,
                    tx_packet.count_us,
                    tx_packet.freq_hz,
                    tx_packet.bandwidth,
                    tx_packet.modulation,
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
    let concentrator_count = timersync::get_concentrator_count();
    let mut queue = queue.lock().unwrap();

    return queue.pop(concentrator_count);
}
