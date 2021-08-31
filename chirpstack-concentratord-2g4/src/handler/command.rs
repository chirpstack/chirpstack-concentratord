use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libconcentratord::signals::Signal;
use libconcentratord::{commands, jitqueue, stats};
use libloragw_2g4::hal;
use prost::Message;
use uuid::Uuid;

use super::super::config::vendor;
use super::super::wrapper;

pub fn handle_loop(
    lorawan_public: bool,
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
                break;
            }
            _ => {}
        };

        let resp = match cmd {
            commands::Command::Timeout => {
                continue;
            }
            commands::Command::Downlink(pl) => {
                match handle_downlink(lorawan_public, vendor_config, gateway_id, &queue, &pl) {
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

    debug!("Command loop ended");
}

fn handle_downlink(
    lorawan_public: bool,
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

    stats::inc_tx_packets_received();

    let mut tx_ack = chirpstack_api::gw::DownlinkTxAck {
        gateway_id: gateway_id.to_vec(),
        token: pl.token,
        downlink_id: pl.downlink_id.to_vec(),
        items: vec![Default::default(); pl.items.len()],
        ..Default::default()
    };
    let mut stats_tx_status = chirpstack_api::gw::TxAckStatus::Ignored;

    for (i, item) in pl.items.iter().enumerate() {
        // convert protobuf to hal struct
        let tx_packet = match wrapper::downlink_from_proto(lorawan_public, item) {
            Ok(v) => v,
            Err(error) => {
                error!(
                    "Convert downlink protobuf to HAL struct error, downlink_id: {}, error: {}",
                    id, error,
                );
                return Err(());
            }
        };

        // validate frequency range
        let freqs = vendor_config.min_max_tx_freq;
        if tx_packet.freq_hz < freqs.0 || tx_packet.freq_hz > freqs.1 {
            error!("Frequency is not within min/max gateway frequency, downlink_id: {}, min_freq: {}, max_freq: {}", id, freqs.0, freqs.1);
            tx_ack.items[i].set_status(chirpstack_api::gw::TxAckStatus::TxFreq);

            // try next
            continue;
        }

        // try enqueue
        match queue.lock().unwrap().enqueue(
            hal::get_instcnt().expect("get concentrator count error"),
            wrapper::TxPacket::new(id, tx_packet),
        ) {
            Ok(_) => {
                tx_ack.items[i].set_status(chirpstack_api::gw::TxAckStatus::Ok);
                stats_tx_status = chirpstack_api::gw::TxAckStatus::Ok;

                // break out of for loop
                break;
            }
            Err(status) => {
                tx_ack.items[i].set_status(status);
                stats_tx_status = status;
            }
        };
    }

    stats::inc_tx_status_count(stats_tx_status);

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
