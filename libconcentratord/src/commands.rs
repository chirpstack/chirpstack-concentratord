use std::time::Duration;

use log::info;
use prost::Message;

use super::socket::ZMQ_CONTEXT;

pub fn get_socket(bind: &str) -> Result<zmq::Socket, zmq::Error> {
    info!("Creating socket for receiving commands, bind: {}", bind);

    let zmq_ctx = ZMQ_CONTEXT.lock().unwrap();
    let sock = zmq_ctx.socket(zmq::REP)?;
    sock.bind(&bind)?;
    return Ok(sock);
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
    pub fn new(sock: &'a zmq::Socket, timeout: Duration) -> Self {
        Reader {
            rep_sock: sock,
            timeout: timeout,
        }
    }
}

impl Iterator for Reader<'_> {
    type Item = Command;

    fn next(&mut self) -> Option<Command> {
        // set poller so that we can timeout
        let mut items = [self.rep_sock.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, self.timeout.as_millis() as i64).unwrap();
        if !items[0].is_readable() {
            return Some(Command::Timeout);
        }

        let msg = self.rep_sock.recv_multipart(0).unwrap();
        match handle_message(msg) {
            Ok(v) => Some(v),
            Err(err) => Some(Command::Error(err.to_string())),
        }
    }
}

fn handle_message(msg: Vec<Vec<u8>>) -> Result<Command, String> {
    if msg.len() != 2 {
        return Err("command must have two frames".to_string());
    }

    let command = match String::from_utf8(msg[0].clone()) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };

    Ok(match command.as_str() {
        "down" => match parse_down(&msg[1]) {
            Ok(v) => Command::Downlink(v),
            Err(err) => Command::Error(err),
        },
        "config" => match parse_config(&msg[1]) {
            Ok(v) => Command::Configuration(v),
            Err(err) => Command::Error(err),
        },
        "gateway_id" => Command::GatewayID,
        _ => Command::Unknown(command, msg[1].clone()),
    })
}

fn parse_down(msg: &[u8]) -> Result<chirpstack_api::gw::DownlinkFrame, String> {
    match chirpstack_api::gw::DownlinkFrame::decode(msg) {
        Ok(v) => Ok(v),
        Err(err) => Err(err.to_string()),
    }
}

fn parse_config(msg: &[u8]) -> Result<chirpstack_api::gw::GatewayConfiguration, String> {
    match chirpstack_api::gw::GatewayConfiguration::decode(msg) {
        Ok(v) => Ok(v),
        Err(err) => Err(err.to_string()),
    }
}
