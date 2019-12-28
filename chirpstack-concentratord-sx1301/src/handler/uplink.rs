use std::{thread, time};

use libloragw_sx1301::hal;
use protobuf::Message;
use uuid::Uuid;

use super::super::wrapper;
use super::stats;

pub fn handle_loop(gateway_id: &[u8], publisher: zmq::Socket) {
    debug!("Starting uplink handle loop");

    loop {
        match hal::receive() {
            Ok(frames) => {
                for frame in frames {
                    let proto = match wrapper::uplink_to_proto(gateway_id.clone(), &frame) {
                        Ok(v) => v,
                        Err(err) => {
                            error!("Convert uplink frame to protobuf error, error: {}", err);
                            continue;
                        }
                    };

                    let uuid = Uuid::from_slice(proto.get_rx_info().get_uplink_id()).unwrap();

                    info!(
                        "Frame received, uplink_id: {}, count_us: {}, freq: {}, bw: {}, mod: {:?}, dr: {:?}",
                        uuid,
                        frame.count_us,
                        frame.freq_hz,
                        frame.bandwidth,
                        frame.modulation,
                        frame.datarate,
                    );

                    stats::inc_rx_packets_received();
                    if proto.get_rx_info().get_crc_status() == chirpstack_api::gw::CRCStatus::CRC_OK
                    {
                        stats::inc_rx_packets_received_ok();
                    }

                    publish_frame(&publisher, proto);
                }
            }
            Err(_) => {
                error!("Receive error");
            }
        };

        thread::sleep(time::Duration::from_millis(10));
    }
}

fn publish_frame(publisher: &zmq::Socket, frame: chirpstack_api::gw::UplinkFrame) {
    let proto_bytes = frame.write_to_bytes().unwrap();
    publisher.send("up", zmq::SNDMORE).unwrap();
    publisher.send(proto_bytes, 0).unwrap();
}
