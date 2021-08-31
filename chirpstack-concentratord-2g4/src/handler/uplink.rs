use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use libconcentratord::signals::Signal;
use libconcentratord::{events, stats};
use libloragw_2g4::hal;
use uuid::Uuid;

use super::super::wrapper;

pub fn handle_loop(gateway_id: &[u8], stop_receive: Receiver<Signal>) {
    debug!("Starting uplink handle loop");

    loop {
        match stop_receive.recv_timeout(Duration::from_millis(0)) {
            Ok(v) => {
                debug!("Received stop signal, signal: {}", v);
                break;
            }
            _ => {}
        };

        match hal::receive() {
            Ok(frames) => {
                for frame in frames {
                    stats::inc_rx_packets_received();
                    if frame.status != hal::CRC::CRCOk {
                        debug!("Frame received with invalid CRC");
                        continue;
                    }

                    let proto = match wrapper::uplink_to_proto(gateway_id.clone(), &frame) {
                        Ok(v) => v,
                        Err(err) => {
                            error!("Convert uplink frame to protobuf error, error: {}", err);
                            continue;
                        }
                    };

                    let rx_info = proto.rx_info.as_ref().unwrap();
                    let uuid = Uuid::from_slice(&rx_info.uplink_id).unwrap();

                    info!(
                        "Frame received, uplink_id: {}, count_us: {}, freq: {}, bw: {}, mod: {:?}, dr: {:?}",
                        uuid,
                        frame.count_us,
                        frame.freq_hz,
                        frame.bandwidth,
                        frame.modulation,
                        frame.datarate,
                    );

                    stats::inc_rx_counts(&proto);
                    events::send_uplink(&proto).unwrap();
                }
            }
            Err(_) => error!("Receive error"),
        };

        thread::sleep(Duration::from_millis(10));
    }

    debug!("Uplink loop ended");
}
