use std::time::Duration;

use anyhow::Result;
use chirpstack_api::{gw, prost::Message};
use log::info;

use crate::error::Error;
use crate::socket::ZMQ_CONTEXT;

pub fn get_socket(bind: &str) -> Result<zmq::Socket> {
    info!("Creating socket for receiving commands, bind: {}", bind);

    let zmq_ctx = ZMQ_CONTEXT.lock().unwrap();
    let sock = zmq_ctx.socket(zmq::REP)?;
    sock.bind(bind)?;
    Ok(sock)
}

pub enum Command {
    // Reading command timed out.
    Timeout,

    // Error reading command.
    Error(String),

    // Unknown command.
    Unknown(String, Vec<u8>),

    // Downlink enqueue.
    Downlink(chirpstack_api::gw::DownlinkFrame),

    // Gateway ID request.
    GatewayID,

    // Gateway configuration.
    Configuration(chirpstack_api::gw::GatewayConfiguration),
}

pub struct Reader<'a> {
    rep_sock: &'a zmq::Socket,
    timeout: Duration,
}

impl<'a> Reader<'a> {
    pub fn new(rep_sock: &'a zmq::Socket, timeout: Duration) -> Self {
        Reader { rep_sock, timeout }
    }
}

impl Iterator for Reader<'_> {
    type Item = Result<gw::Command, Error>;

    fn next(&mut self) -> Option<Result<gw::Command, Error>> {
        // set poller so that we can timeout
        let mut items = [self.rep_sock.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, self.timeout.as_millis() as i64).unwrap();
        if !items[0].is_readable() {
            return Some(Err(Error::Timeout));
        }

        let b = self.rep_sock.recv_bytes(0).unwrap();
        match gw::Command::decode(b.as_slice()).map_err(|e| Error::Anyhow(anyhow::Error::new(e))) {
            Ok(v) => Some(Ok(v)),
            Err(e) => Some(Err(e)),
        }
    }
}
