use std::sync::Mutex;

use log::info;
use prost::Message;
use uuid::Uuid;

use super::socket::ZMQ_CONTEXT;

lazy_static! {
    static ref ZMQ_PUB: Mutex<Option<zmq::Socket>> = Mutex::new(None);
}

pub fn bind_socket(bind: &str) -> Result<(), zmq::Error> {
    info!("Creating socket for publishing events, bind: {}", bind);

    let zmq_ctx = ZMQ_CONTEXT.lock().unwrap();
    let mut zmq_pub = ZMQ_PUB.lock().unwrap();

    let sock = zmq_ctx.socket(zmq::PUB)?;
    sock.bind(&bind)?;

    *zmq_pub = Some(sock);

    return Ok(());
}

pub fn send_uplink(pl: &chirpstack_api::gw::UplinkFrame) -> Result<(), String> {
    let pub_guard = ZMQ_PUB.lock().unwrap();
    let publisher = pub_guard.as_ref().unwrap();

    let mut buf = Vec::new();
    pl.encode(&mut buf).unwrap();
    publisher.send("up", zmq::SNDMORE).unwrap();
    publisher.send(buf, 0).unwrap();

    return Ok(());
}

pub fn send_stats(stats: &chirpstack_api::gw::GatewayStats, stats_id: &Uuid) -> Result<(), String> {
    let pub_guard = ZMQ_PUB.lock().unwrap();
    let publisher = pub_guard.as_ref().unwrap();

    info!("Publishing stats event, stats_id: {}, rx_received: {}, rx_received_ok: {}, tx_received: {}, tx_emitted: {}", stats_id, stats.rx_packets_received, stats.rx_packets_received_ok, stats.tx_packets_received, stats.tx_packets_emitted);

    let mut buf = Vec::new();
    stats.encode(&mut buf).unwrap();
    publisher.send("stats", zmq::SNDMORE).unwrap();
    publisher.send(buf, 0).unwrap();

    return Ok(());
}
