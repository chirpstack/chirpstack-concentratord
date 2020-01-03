use std::sync::{Arc, Mutex};

use libconcentratord::{commands, jitqueue, stats};
use libloragw_sx1302::hal;
use protobuf::Message;
use uuid::Uuid;

use super::super::config::vendor;
use super::super::wrapper;

pub fn handle_loop(
    vendor_config: &vendor::Configuration,
    gateway_id: &[u8],
    queue: Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
    rep_sock: zmq::Socket,
) {
    debug!("Starting command handler loop");

    let reader = commands::Reader::new(&rep_sock);

    for cmd in reader {
        let resp = match cmd {
            commands::Command::Downlink(pl) => {
                match handle_downlink(vendor_config, gateway_id, &queue, &pl) {
                    Ok(v) => v,
                    Err(_) => Vec::new(),
                }
            }
            commands::Command::GatewayID => gateway_id.to_vec(),
            commands::Command::Error(err) => {
                error!("Read command error, error: {}", err);
                Vec::new()
            }
            commands::Command::Unknown(command, _) => {
                warn!("Unknown command received, command: {}", command);
                Vec::new()
            }
        };

        rep_sock.send(resp, 0).unwrap();
    }
}

fn handle_downlink(
    vendor_config: &vendor::Configuration,
    gateway_id: &[u8],
    queue: &Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
    pl: &chirpstack_api::gw::DownlinkFrame,
) -> Result<Vec<u8>, ()> {
    let id = match Uuid::from_slice(pl.get_downlink_id()) {
        Ok(v) => v,
        Err(err) => {
            error!("Decode downlink_id error: {}", err);
            return Err(());
        }
    };

    let tx_packet = match wrapper::downlink_from_proto(&pl) {
        Ok(v) => v,
        Err(err) => {
            error!(
                "Convert downlink protobuf to HAL struct error, downlink_id: {}, error: {}",
                id, err,
            );
            return Err(());
        }
    };

    stats::inc_tx_packets_received();

    let mut valid = true;
    let mut tx_ack = chirpstack_api::gw::DownlinkTXAck::default();
    tx_ack.set_token(pl.get_token());
    tx_ack.set_downlink_id(pl.get_downlink_id().to_vec());
    tx_ack.set_gateway_id(gateway_id.to_vec());

    match vendor_config.radio_config.get(tx_packet.rf_chain as usize) {
        Some(v) => {
            if tx_packet.freq_hz < v.tx_freq_min || tx_packet.freq_hz > v.tx_freq_max {
                valid = false;
                error!("Frequency is not within min/max gateway frequency, downlink_id: {}, min_freq: {}, max_freq: {}", id, v.tx_freq_min, v.tx_freq_max);
                tx_ack.set_error("TX_FREQ".to_string());
            }
        }
        None => {
            valid = false;
            tx_ack.set_error("RF_CHAIN".to_string());
        }
    }

    if valid {
        match queue.lock().unwrap().enqueue(
            hal::get_instcnt().expect("get concentrator count error"),
            wrapper::TxPacket::new(id, tx_packet),
        ) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Enqueue downlink error, downlink_id: {}, error: {:?}",
                    id, err
                );

                match err {
                    jitqueue::EnqueueError::Collision => {
                        tx_ack.set_error("COLLISION_PACKET".to_string())
                    }
                    jitqueue::EnqueueError::FullQueue => tx_ack.set_error("QUEUE_FULL".to_string()),
                    jitqueue::EnqueueError::TooLate => tx_ack.set_error("TOO_LATE".to_string()),
                    jitqueue::EnqueueError::TooEarly => tx_ack.set_error("TOO_EARLY".to_string()),
                    jitqueue::EnqueueError::Unknown(err) => tx_ack.set_error(err),
                }
            }
        };
    }

    let tx_ack_bytes = tx_ack.write_to_bytes().unwrap();
    return Ok(tx_ack_bytes);
}
