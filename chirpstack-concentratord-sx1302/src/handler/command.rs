use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libconcentratord::signals::Signal;
use libconcentratord::{commands, jitqueue, stats};
use libloragw_sx1302::hal;
use prost::Message;
use uuid::Uuid;

use super::super::config::vendor;
use super::super::wrapper;

pub fn handle_loop(
    vendor_config: &vendor::Configuration,
    gateway_id: &[u8],
    queue: Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
    rep_sock: zmq::Socket,
    stop_receive: Receiver<Signal>,
    stop_send: Sender<Signal>,
) {
    debug!("Starting command handler loop");

    // A timeout is used so that we can consume from the stop signal.
    let reader = commands::Reader::new(&rep_sock, Duration::from_millis(100));

    for cmd in reader {
        match stop_receive.recv_timeout(Duration::from_millis(0)) {
            Ok(v) => {
                debug!("Received stop signal, signal: {}", v);
                return;
            }
            _ => {}
        };

        let resp = match cmd {
            commands::Command::Timeout => {
                continue;
            }
            commands::Command::Downlink(pl) => {
                match handle_downlink(vendor_config, gateway_id, &queue, &pl) {
                    Ok(v) => v,
                    Err(_) => Vec::new(),
                }
            }
            commands::Command::GatewayID => gateway_id.to_vec(),
            commands::Command::Configuration(pl) => {
                match handle_configuration(stop_send.clone(), pl) {
                    Ok(v) => v,
                    Err(_) => Vec::new(),
                }
            }
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
    let id = match Uuid::from_slice(&pl.downlink_id) {
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
    let mut tx_ack = chirpstack_api::gw::DownlinkTxAck::default();
    tx_ack.token = pl.token;
    tx_ack.downlink_id = pl.downlink_id.to_vec();
    tx_ack.gateway_id = gateway_id.to_vec();

    match vendor_config.radio_config.get(tx_packet.rf_chain as usize) {
        Some(v) => {
            if tx_packet.freq_hz < v.tx_freq_min || tx_packet.freq_hz > v.tx_freq_max {
                valid = false;
                error!("Frequency is not within min/max gateway frequency, downlink_id: {}, min_freq: {}, max_freq: {}", id, v.tx_freq_min, v.tx_freq_max);
                tx_ack.error = "TX_FREQ".to_string();
            }
        }
        None => {
            valid = false;
            tx_ack.error = "RF_CHAIN".to_string();
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
                        tx_ack.error = "COLLISION_PACKET".to_string()
                    }
                    jitqueue::EnqueueError::FullQueue => tx_ack.error = "QUEUE_FULL".to_string(),
                    jitqueue::EnqueueError::TooLate => tx_ack.error = "TOO_LATE".to_string(),
                    jitqueue::EnqueueError::TooEarly => tx_ack.error = "TOO_EARLY".to_string(),
                    jitqueue::EnqueueError::Unknown(err) => tx_ack.error = err,
                }
            }
        };
    }

    let mut buf = Vec::new();
    tx_ack.encode(&mut buf).unwrap();
    return Ok(buf);
}

fn handle_configuration(
    stop_send: Sender<Signal>,
    pl: chirpstack_api::gw::GatewayConfiguration,
) -> Result<Vec<u8>, ()> {
    stop_send.send(Signal::Configuration(pl)).unwrap();
    return Ok(Vec::new());
}
