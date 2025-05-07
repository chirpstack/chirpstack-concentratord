use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};

use libconcentratord::signals::Signal;
use libconcentratord::{events, stats};
use libloragw_2g4::hal;

use crate::wrapper;

pub fn handle_loop(
    gateway_id: &[u8],
    stop_receive: Receiver<Signal>,
    disable_crc_filter: bool,
    time_fallback: bool,
) -> Result<()> {
    debug!("Starting uplink handle loop");

    loop {
        if let Ok(v) = stop_receive.recv_timeout(Duration::from_millis(0)) {
            debug!("Received stop signal, signal: {}", v);
            return Ok(());
        }

        match hal::receive() {
            Ok(frames) => {
                for frame in frames {
                    stats::inc_rx_packets_received();

                    if !disable_crc_filter && frame.status != hal::CRC::CRCOk {
                        debug!(
                            "Frame received with invalid CRC, see disable_crc_filter configuration option if you want to receive these frames"
                        );
                        continue;
                    }

                    let proto = match wrapper::uplink_to_proto(gateway_id, &frame, time_fallback) {
                        Ok(v) => v,
                        Err(err) => {
                            error!("Convert uplink frame to protobuf error, error: {}", err);
                            continue;
                        }
                    };

                    let rx_info = proto
                        .rx_info
                        .as_ref()
                        .ok_or_else(|| anyhow!("rx_info is None"))?;

                    info!(
                        "Frame received, uplink_id: {}, count_us: {}, freq: {}, bw: {}, mod: {:?}, dr: {:?}",
                        rx_info.uplink_id,
                        frame.count_us,
                        frame.freq_hz,
                        frame.bandwidth,
                        frame.modulation,
                        frame.datarate,
                    );

                    if frame.status == hal::CRC::CRCOk {
                        stats::inc_rx_counts(&proto);
                    }
                    events::send_uplink(proto).context("Send uplink")?;
                }
            }
            Err(_) => error!("Receive error"),
        };

        thread::sleep(Duration::from_millis(10));
    }
}
