use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use libconcentratord::signals::Signal;
use libconcentratord::{events, stats};
use libloragw_sx1302::hal;

use super::super::wrapper;

pub fn handle_loop(
    gateway_id: &[u8],
    stop_receive: Receiver<Signal>,
    disable_crc_filter: bool,
    time_fallback: bool,
) {
    debug!("Starting uplink handle loop");

    loop {
        if let Ok(v) = stop_receive.recv_timeout(Duration::from_millis(0)) {
            debug!("Received stop signal, signal: {}", v);
            break;
        }

        match hal::receive() {
            Ok(frames) => {
                for frame in frames {
                    stats::inc_rx_packets_received();

                    if !disable_crc_filter && frame.status != hal::CRC::CRCOk {
                        debug!("Frame received with invalid CRC, see disable_crc_filter configuration option if you want to receive these frames");
                        continue;
                    }

                    let proto = match wrapper::uplink_to_proto(gateway_id, &frame, time_fallback) {
                        Ok(v) => v,
                        Err(err) => {
                            error!("Convert uplink frame to protobuf error, error: {}", err);
                            continue;
                        }
                    };

                    let rx_info = proto.rx_info.as_ref().unwrap();

                    info!(
                        "Frame received, uplink_id: {}, count_us: {}, freq: {}, bw: {}, mod: {:?}, dr: {:?}, ftime_received: {}, ftime_ns: {}",
                        rx_info.uplink_id,
                        frame.count_us,
                        frame.freq_hz,
                        frame.bandwidth,
                        frame.modulation,
                        frame.datarate,
                        frame.ftime_received,
                        frame.ftime,
                    );

                    if frame.status == hal::CRC::CRCOk {
                        stats::inc_rx_counts(&proto);
                    }
                    events::send_uplink(&proto).unwrap();
                }
            }
            Err(_) => error!("Receive error"),
        };

        thread::sleep(Duration::from_millis(10));
    }

    debug!("Uplink loop ended");
}
