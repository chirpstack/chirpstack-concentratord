use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use chirpstack_api::{common, gw, prost::Message};
use libconcentratord::signals::Signal;
use libconcentratord::{commands, jitqueue, stats};
use libloragw_2g4::hal;

use crate::{config::vendor, handler::gps, wrapper};

#[allow(clippy::too_many_arguments)]
pub fn handle_loop(
    lorawan_public: bool,
    vendor_config: &vendor::Configuration,
    gateway_id: &[u8],
    queue: Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
    rep_sock: zmq::Socket,
    stop_receive: Receiver<Signal>,
    stop_send: Sender<Signal>,
) -> Result<()> {
    debug!("Starting command handler loop");

    // A timeout is used so that we can consume from the stop signal.
    let reader = commands::Reader::new(&rep_sock, Duration::from_millis(100));

    for cmd in reader {
        if let Ok(v) = stop_receive.recv_timeout(Duration::from_millis(0)) {
            debug!("Received stop signal, signal: {}", v);
            return Ok(());
        }

        let resp = match cmd {
            Ok(v) => match v.command {
                Some(gw::command::Command::SendDownlinkFrame(v)) => {
                    handle_downlink(lorawan_public, vendor_config, gateway_id, &queue, &v)
                        .unwrap_or_else(|e| {
                            error!("Handle downlink error, error: {}", e);
                            Vec::new()
                        })
                }
                Some(gw::command::Command::SetGatewayConfiguration(v)) => {
                    handle_configuration(stop_send.clone(), v).unwrap_or_else(|e| {
                        error!("Handle configuration error, error: {}", e);
                        Vec::new()
                    })
                }
                Some(gw::command::Command::GetGatewayId(_)) => {
                    let resp = gw::GetGatewayIdResponse {
                        gateway_id: hex::encode(gateway_id),
                    };
                    resp.encode_to_vec()
                }
                Some(gw::command::Command::GetLocation(_)) => gw::GetLocationResponse {
                    location: gps::get_coords().map(|v| common::Location {
                        latitude: v.latitude,
                        longitude: v.longitude,
                        altitude: v.altitude.into(),
                        source: common::LocationSource::Gps.into(),
                        ..Default::default()
                    }),
                    updated_at: None,
                }
                .encode_to_vec(),
                _ => Vec::new(),
            },
            Err(e) => match e {
                libconcentratord::error::Error::Timeout => continue,
                _ => {
                    warn!("Read command error, error: {}", e);
                    Vec::new()
                }
            },
        };

        rep_sock.send(resp, 0)?;
    }

    Ok(())
}

fn handle_downlink(
    lorawan_public: bool,
    vendor_config: &vendor::Configuration,
    gateway_id: &[u8],
    queue: &Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
    pl: &chirpstack_api::gw::DownlinkFrame,
) -> Result<Vec<u8>> {
    stats::inc_tx_packets_received();

    let mut tx_ack = chirpstack_api::gw::DownlinkTxAck {
        gateway_id: hex::encode(gateway_id),
        downlink_id: pl.downlink_id,
        items: vec![Default::default(); pl.items.len()],
        ..Default::default()
    };
    let mut stats_tx_status = chirpstack_api::gw::TxAckStatus::Ignored;

    for (i, item) in pl.items.iter().enumerate() {
        // convert protobuf to hal struct
        let tx_packet = match wrapper::downlink_from_proto(lorawan_public, item) {
            Ok(v) => v,
            Err(err) => {
                error!(
                    "Convert downlink protobuf to HAL struct error, downlink_id: {}, error: {}",
                    pl.downlink_id, err,
                );
                return Err(err);
            }
        };

        // validate frequency range
        if !vendor_config
            .tx_min_max_freqs
            .iter()
            .map(|(freq_min, freq_max)| {
                tx_packet.freq_hz >= *freq_min && tx_packet.freq_hz <= *freq_max
            })
            .collect::<Vec<bool>>()
            .contains(&true)
        {
            error!(
                "Frequency is not within min / max gateway frequencies, downlink_id: {}, freq: {}",
                pl.downlink_id, tx_packet.freq_hz
            );
            tx_ack.items[i].set_status(chirpstack_api::gw::TxAckStatus::TxFreq);

            // try next
            continue;
        }

        // try enqueue
        match queue
            .lock()
            .map_err(|_| anyhow!("Queue lock error"))?
            .enqueue(
                hal::get_instcnt()?,
                wrapper::TxPacket::new(pl.downlink_id, tx_packet),
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

    Ok(tx_ack.encode_to_vec())
}

fn handle_configuration(
    stop_send: Sender<Signal>,
    pl: chirpstack_api::gw::GatewayConfiguration,
) -> Result<Vec<u8>> {
    stop_send.send(Signal::Configuration(pl)).unwrap();
    Ok(Vec::new())
}
