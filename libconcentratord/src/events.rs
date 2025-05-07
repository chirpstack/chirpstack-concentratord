use std::sync::{LazyLock, Mutex};

use anyhow::Result;
use chirpstack_api::{gw, prost::Message};
use log::info;

use super::socket::ZMQ_CONTEXT;

static ZMQ_PUB: LazyLock<Mutex<Option<zmq::Socket>>> = LazyLock::new(|| Mutex::new(None));

pub fn bind_socket(bind: &str) -> Result<()> {
    info!("Creating socket for publishing events, bind: {}", bind);

    let zmq_ctx = ZMQ_CONTEXT.lock().unwrap();
    let mut zmq_pub = ZMQ_PUB.lock().unwrap();

    let sock = zmq_ctx.socket(zmq::PUB)?;
    sock.bind(bind)?;

    *zmq_pub = Some(sock);

    Ok(())
}

pub fn send_uplink(pl: chirpstack_api::gw::UplinkFrame) -> Result<()> {
    let pub_guard = ZMQ_PUB.lock().unwrap();
    let publisher = pub_guard.as_ref().unwrap();

    let event = gw::Event {
        event: Some(gw::event::Event::UplinkFrame(pl)),
    };

    publisher.send(event.encode_to_vec(), 0).unwrap();

    Ok(())
}

pub fn send_stats(stats: chirpstack_api::gw::GatewayStats) -> Result<()> {
    let pub_guard = ZMQ_PUB.lock().unwrap();
    let publisher = pub_guard.as_ref().unwrap();

    info!(
        "Publishing stats event, rx_received: {}, rx_received_ok: {}, tx_received: {}, tx_emitted: {}",
        stats.rx_packets_received,
        stats.rx_packets_received_ok,
        stats.tx_packets_received,
        stats.tx_packets_emitted
    );

    let event = gw::Event {
        event: Some(gw::event::Event::GatewayStats(stats)),
    };

    publisher.send(event.encode_to_vec(), 0).unwrap();

    Ok(())
}
