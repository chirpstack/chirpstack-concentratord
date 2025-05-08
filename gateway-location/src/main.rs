use std::process::exit;

use chirpstack_api::{gw, prost::Message};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// ZMQ command URL
    #[arg(short, long, default_value = "ipc:///tmp/concentratord_command")]
    command_url: String,
}

fn main() {
    let cli = Cli::parse();

    // create new zmq REQ socket
    let zmq_ctx = zmq::Context::new();
    let zmq_sock = zmq_ctx.socket(zmq::REQ).expect("new ZMQ socket error");
    zmq_sock
        .connect(&cli.command_url)
        .expect("ZMQ connect error");

    // Send command.
    let cmd = gw::Command {
        command: Some(gw::command::Command::GetLocation(gw::GetLocationRequest {})),
    };
    zmq_sock.send(cmd.encode_to_vec(), 0).unwrap();

    // set poller so that we can timeout after 100ms
    let mut items = [zmq_sock.as_poll_item(zmq::POLLIN)];
    zmq::poll(&mut items, 100).unwrap();
    if !items[0].is_readable() {
        println!("could not read gateway_id");
        exit(1);
    }

    // Read response.
    let b = zmq_sock.recv_bytes(0).unwrap();
    let resp = gw::GetLocationResponse::decode(b.as_slice()).unwrap();
    println!("{:?}", resp);
    exit(0);
}
