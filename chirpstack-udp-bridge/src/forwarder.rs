use std::net::UdpSocket;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::{thread, time};

use prost::Message;
use rand::Rng;

use super::config::Server;
use super::metrics;
use super::signals;
use super::structs;
use libconcentratord::{commands, events};

struct State {
    server: String,
    keepalive_interval: time::Duration,
    keepalive_max_failures: u32,
    gateway_id: Vec<u8>,
    socket: UdpSocket,
    push_data_token: Mutex<u16>,
    push_data_sent: Mutex<u32>,
    push_data_acked: Mutex<u32>,
    pull_data_token: Mutex<u16>,
    pull_data_token_acked: Mutex<u16>,
    rxfw: Mutex<u32>,
    event_sock: Mutex<zmq::Socket>,
    command_sock: Mutex<zmq::Socket>,
}

impl State {
    fn set_pull_data_token(&self) -> u16 {
        let mut rng = rand::thread_rng();
        let mut token = self.pull_data_token.lock().unwrap();
        *token = rng.gen();
        return *token;
    }

    fn get_pull_data_token(&self) -> u16 {
        return *self.pull_data_token.lock().unwrap();
    }

    fn get_pull_data_token_acked(&self) -> u16 {
        *self.pull_data_token_acked.lock().unwrap()
    }

    fn set_pull_data_token_acked(&self, t: u16) {
        let mut token = self.pull_data_token_acked.lock().unwrap();
        *token = t
    }

    fn set_push_data_token(&self) -> u16 {
        let mut rng = rand::thread_rng();
        let mut token = self.push_data_token.lock().unwrap();
        *token = rng.gen();
        return *token;
    }

    fn get_push_data_token(&self) -> u16 {
        return *self.push_data_token.lock().unwrap();
    }

    fn incr_push_data_sent(&self) {
        let mut sent = self.push_data_sent.lock().unwrap();
        *sent = *sent + 1;
    }

    fn get_and_reset_push_data_sent(&self) -> u32 {
        let mut sent = self.push_data_sent.lock().unwrap();
        let out = *sent;
        *sent = 0;
        out
    }

    fn incr_push_data_acked(&self) {
        let mut acked = self.push_data_acked.lock().unwrap();
        *acked = *acked + 1;
    }

    fn get_and_reset_push_data_acked(&self) -> u32 {
        let mut acked = self.push_data_acked.lock().unwrap();
        let out = *acked;
        *acked = 0;
        out
    }

    fn incr_rxfw(&self) {
        let mut rxfw = self.rxfw.lock().unwrap();
        *rxfw = *rxfw + 1;
    }

    fn get_and_reset_rxfw(&self) -> u32 {
        let mut rxfw = self.rxfw.lock().unwrap();
        let out = *rxfw;
        *rxfw = 0;
        return out;
    }
}

pub fn start(conf: &Server, event_url: String, command_url: String, gateway_id: Vec<u8>) {
    // loop so that we can restart the forwarder
    loop {
        info!("Starting forwarder, server: {}", conf.server);

        // setup udp socket
        let socket = UdpSocket::bind("0.0.0.0:0").expect("udp socket bind error");
        socket
            .connect(&conf.server)
            .expect("connect udp socket error");
        socket
            .set_read_timeout(Some(time::Duration::from_millis(100)))
            .unwrap();

        // setup state
        let state = State {
            server: conf.server.clone(),
            keepalive_interval: match conf.keepalive_interval_secs {
                0 => time::Duration::from_secs(5),
                _ => time::Duration::from_secs(conf.keepalive_interval_secs),
            },
            keepalive_max_failures: conf.keepalive_max_failures,
            gateway_id: gateway_id.clone(),
            socket: socket,
            push_data_token: Mutex::new(0),
            push_data_sent: Mutex::new(0),
            push_data_acked: Mutex::new(0),
            pull_data_token: Mutex::new(0),
            pull_data_token_acked: Mutex::new(0),
            rxfw: Mutex::new(0),
            event_sock: Mutex::new(
                events::get_client(&event_url).expect("get events client error"),
            ),
            command_sock: Mutex::new(
                commands::get_client(&command_url).expect("get commands client error"),
            ),
        };
        let state = Arc::new(state);

        // Signal pool so that we can stop all threads in case of x failed
        // keepalive frames and start over again.
        let mut signal_pool = signals::SignalPool::new();

        // setup threads
        // let mut signal_pool = signals::SignalPool::new();
        let mut threads: Vec<thread::JoinHandle<()>> = vec![];

        // UDP receive loop
        threads.push(thread::spawn({
            let state = state.clone();
            let stop_receive = signal_pool.new_receiver();

            move || {
                udp_receive_loop(state, stop_receive);
            }
        }));

        // event thread.
        threads.push(thread::spawn({
            let state = state.clone();
            let stop_receive = signal_pool.new_receiver();

            move || {
                events_loop(state, stop_receive);
            }
        }));

        // PULL_DATA thread.
        threads.push(thread::spawn({
            let state = state.clone();

            move || {
                pull_data_loop(state, signal_pool);
            }
        }));

        for t in threads {
            t.join().unwrap();
        }

        warn!("Forwarder stopped, server: {}", conf.server);
    }
}

fn pull_data_loop(state: Arc<State>, signal_pool: signals::SignalPool) {
    let mut missed_acks: u32 = 0;

    loop {
        if state.get_pull_data_token() != state.get_pull_data_token_acked() {
            warn!(
                "Server did not acknowledge PULL_DATA, server: {}, token: {}",
                state.server,
                state.get_pull_data_token()
            );
            missed_acks = missed_acks + 1;
        } else {
            missed_acks = 0;
        }

        if state.keepalive_max_failures != 0 && missed_acks > state.keepalive_max_failures {
            warn!(
                "Max missed keepalive frames missed, server: {}",
                state.server
            );
            signal_pool.send_signal(signals::Signal::Stop);

            debug!("Terminating PULL_DATA loop, server: {}", state.server);
            return;
        }

        let mut id: [u8; 8] = [0; 8];
        id.copy_from_slice(&state.gateway_id);

        let pull_data = structs::PullData {
            gateway_id: id,
            random_token: state.set_pull_data_token(),
        };
        let bytes = pull_data.to_bytes();

        info!("Sending PULL_DATA to server, server: {}", state.server);
        if let Err(e) = state.socket.send(&bytes) {
            error!("UDP send error: {}, server: {}", e, state.server);
        };

        metrics::incr_udp_sent_count(&state.server, "PULL_DATA");
        metrics::incr_udp_sent_bytes(&state.server, "PULL_DATA", bytes.len());

        thread::sleep(state.keepalive_interval);
    }
}

fn udp_receive_loop(state: Arc<State>, stop_receive: Receiver<signals::Signal>) {
    let mut buffer: [u8; 65535] = [0; 65535];

    loop {
        if let Ok(_) = stop_receive.recv_timeout(time::Duration::from_millis(0)) {
            debug!("Terminating UDP receive loop, server: {}", state.server);
            return;
        };

        let size = match state.socket.recv(&mut buffer) {
            Ok(v) => v,
            Err(_) => {
                // Most likely, a timeout occured.
                continue;
            }
        };

        if size < 4 {
            warn!(
                "At least 4 bytes are expected, received: {}, server: {}",
                size, state.server
            );
            continue;
        }

        match buffer[3] {
            0x01 => {
                metrics::incr_udp_received_count(&state.server, "PUSH_ACK");
                metrics::incr_udp_received_bytes(&state.server, "PUSH_ACK", size);

                if let Err(e) = handle_push_ack(&state, &buffer[..size]) {
                    warn!("Handling PUSH_ACK error: {}, server: {}", e, state.server);
                };
            }
            0x03 => {
                metrics::incr_udp_received_count(&state.server, "PULL_RESP");
                metrics::incr_udp_received_bytes(&state.server, "PULL_RESP", size);

                if let Err(e) = handle_pull_resp(&state, &buffer[..size]) {
                    warn!("handling PULL_RESP error: {}, server: {}", e, state.server);
                };
            }
            0x04 => {
                metrics::incr_udp_received_count(&state.server, "PULL_ACK");
                metrics::incr_udp_received_bytes(&state.server, "PULL_ACK", size);

                if let Err(e) = handle_pull_ack(&state, &buffer[..size]) {
                    warn!("Handling PULL_ACK error: {}, server: {}", e, state.server);
                };
            }
            _ => {
                metrics::incr_udp_received_count(&state.server, "UNKNOWN");
                metrics::incr_udp_received_bytes(&state.server, "UNKNOWN", size);

                warn!(
                    "Ignoring unexepcted command, cid: {}, server: {}",
                    buffer[3], state.server
                );
                continue;
            }
        }
    }
}

fn events_loop(state: Arc<State>, stop_receive: Receiver<signals::Signal>) {
    let event_sock = state.event_sock.lock().unwrap();
    let reader = events::Reader::new(&event_sock, time::Duration::from_millis(100));

    for cmd in reader {
        if let Ok(_) = stop_receive.recv_timeout(time::Duration::from_millis(0)) {
            debug!("Terminating events loop, server: {}", state.server);
            return;
        }

        match cmd {
            events::Event::Uplink(up) => {
                events_up(&state, up);
            }
            events::Event::Stats(stats) => {
                events_stats(&state, stats);
            }
            events::Event::Timeout => {
                continue;
            }
            events::Event::Error(err) => {
                error!("Read event error, error: {}", err);
            }
            events::Event::Unknown(event, _) => {
                warn!("Unknown event received, event: {}", event);
            }
        }
    }
}

fn events_stats(state: &Arc<State>, stats: chirpstack_api::gw::GatewayStats) {
    let mut stat = match structs::Stat::from_proto(&stats) {
        Ok(v) => v,
        Err(err) => {
            error!("Stats from proto message error: {}", err);
            return;
        }
    };
    stat.rxfw = state.get_and_reset_rxfw();

    let pd_sent = state.get_and_reset_push_data_sent();
    let pd_acked = state.get_and_reset_push_data_acked();
    if pd_sent != 0 {
        stat.ackr = pd_acked as f32 / pd_sent as f32 * 100.0
    }

    let mut id: [u8; 8] = [0; 8];
    id.copy_from_slice(&state.gateway_id);

    let push_data = structs::PushData {
        random_token: state.set_push_data_token(),
        gateway_id: id,
        payload: structs::PushDataPayload {
            stat: Some(stat),
            rxpk: vec![],
        },
    };
    let bytes = push_data.to_bytes();

    info!(
        "Sending PUSH_DATA with stats to server, server: {}",
        state.server
    );
    if let Err(e) = state.socket.send(&bytes) {
        error!("UDP send error: {}, server: {}", e, state.server);
    };

    state.incr_push_data_sent();

    metrics::incr_udp_sent_count(&state.server, "PUSH_DATA_STATS");
    metrics::incr_udp_sent_bytes(&state.server, "PUSH_DATA_STATS", bytes.len());
}

fn events_up(state: &Arc<State>, up: chirpstack_api::gw::UplinkFrame) {
    let rxpk = match structs::RXPK::from_proto(&up) {
        Ok(v) => v,
        Err(err) => {
            error!("RXPK from proto message error: {}", err);
            return;
        }
    };

    let mut id: [u8; 8] = [0; 8];
    id.copy_from_slice(&state.gateway_id);

    let push_data = structs::PushData {
        random_token: state.set_push_data_token(),
        gateway_id: id,
        payload: structs::PushDataPayload {
            stat: None,
            rxpk: vec![rxpk],
        },
    };
    let bytes = push_data.to_bytes();

    info!(
        "Sending PUSH_DATA with rxpk to server, server: {}",
        state.server
    );
    if let Err(e) = state.socket.send(&bytes) {
        error!("UDP send error: {}, server: {}", e, state.server);
    };

    state.incr_rxfw();
    state.incr_push_data_sent();

    metrics::incr_udp_sent_count(&state.server, "PUSH_DATA_RXPK");
    metrics::incr_udp_sent_bytes(&state.server, "PUSH_DATA_RXPK", bytes.len());
}

fn handle_push_ack(state: &Arc<State>, data: &[u8]) -> Result<(), String> {
    let push_ack = structs::PushAck::from_bytes(&data)?;
    let expected_token = state.get_push_data_token();

    if push_ack.random_token == expected_token {
        debug!(
            "PUSH_DATA acknowledged, token: {}, server: {}",
            expected_token, state.server
        );

        state.incr_push_data_acked();
    }

    Ok(())
}

fn handle_pull_ack(state: &Arc<State>, data: &[u8]) -> Result<(), String> {
    let push_ack = structs::PullAck::from_bytes(data)?;
    let expected_token = state.get_pull_data_token();
    state.set_pull_data_token_acked(push_ack.random_token);

    if push_ack.random_token == expected_token {
        info!(
            "PULL_DATA acknowledged, token: {}, server: {}",
            expected_token, state.server
        );
    }

    Ok(())
}

fn handle_pull_resp(state: &Arc<State>, data: &[u8]) -> Result<(), String> {
    let pull_resp = structs::PullResp::from_bytes(data)?;
    let sock = state.command_sock.lock().unwrap();

    // the UDP token is 2 bytes, an UUID is 16, therefore we prefix the
    // token with 14 empty bytes.
    let mut downlink_id = vec![0; 14];
    downlink_id.append(&mut pull_resp.random_token.to_be_bytes().to_vec());

    let pl = match pull_resp
        .payload
        .txpk
        .to_proto(downlink_id, state.gateway_id.clone())
    {
        Ok(v) => v,
        Err(err) => {
            return Err(format!("TXPK to proto error: {}", err).to_string());
        }
    };

    let mut buf = Vec::new();
    pl.encode(&mut buf).unwrap();

    // send 'down' command with payload
    sock.send("down", zmq::SNDMORE).unwrap();
    sock.send(buf, 0).unwrap();

    // set poller so that we can timeout after 100ms
    let mut items = [sock.as_poll_item(zmq::POLLIN)];
    zmq::poll(&mut items, 100).unwrap();
    if !items[0].is_readable() {
        return Err("could not read down response".to_string());
    }

    // read tx ack response.
    let resp_b: &[u8] = &sock.recv_bytes(0).unwrap();
    let tx_ack = match chirpstack_api::gw::DownlinkTxAck::decode(resp_b) {
        Ok(v) => v,
        Err(err) => {
            return Err(format!("decode DownlinkTxAck error: {}", err).to_string());
        }
    };

    // udp tx ack
    let tx_ack_udp = structs::TxAck {
        random_token: pull_resp.random_token,
        gateway_id: {
            let mut id: [u8; 8] = [0; 8];
            id.copy_from_slice(&state.gateway_id);
            id
        },
        payload: structs::TxAckPayload {
            txpk_ack: structs::TxAckPayloadError {
                error: {
                    if tx_ack.items.len() != 1 {
                        return Err("".to_string());
                    }

                    match tx_ack.items[0].status() {
                        chirpstack_api::gw::TxAckStatus::Ok => "".to_string(),
                        chirpstack_api::gw::TxAckStatus::Ignored => "IGNORED".to_string(),
                        chirpstack_api::gw::TxAckStatus::TooLate => "TOO_LATE".to_string(),
                        chirpstack_api::gw::TxAckStatus::TooEarly => "TOO_EARLY".to_string(),
                        chirpstack_api::gw::TxAckStatus::CollisionPacket => {
                            "COLLISION_PACKET".to_string()
                        }
                        chirpstack_api::gw::TxAckStatus::CollisionBeacon => {
                            "COLLISION_BEACON".to_string()
                        }
                        chirpstack_api::gw::TxAckStatus::TxFreq => "TX_FREQ".to_string(),
                        chirpstack_api::gw::TxAckStatus::TxPower => "TX_POWER".to_string(),
                        chirpstack_api::gw::TxAckStatus::GpsUnlocked => "GPS_UNLOCKED".to_string(),
                        chirpstack_api::gw::TxAckStatus::QueueFull => "QUEUE_FULL".to_string(),
                        chirpstack_api::gw::TxAckStatus::InternalError => {
                            "INTERNAL_ERROR".to_string()
                        }
                    }
                },
            },
        },
    };
    let bytes = tx_ack_udp.to_bytes();

    debug!("Sending TX_ACK to server, server: {}", state.server);
    if let Err(e) = state.socket.send(&bytes) {
        error!("UDP send error: {}, server: {}", e, state.server);
    };

    metrics::incr_udp_sent_count(&state.server, "TX_ACK");
    metrics::incr_udp_sent_bytes(&state.server, "TX_ACK", bytes.len());

    Ok(())
}
