use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use prometheus::{Encoder, IntCounterVec, Opts, Registry};

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();

    // UDP sent
    static ref UDP_SENT_COUNT: IntCounterVec = IntCounterVec::new(Opts::new("udp_sent_count", "Number of UDP datagrams sent"), &["server", "type"]).unwrap();
    static ref UDP_SENT_BYTES: IntCounterVec = IntCounterVec::new(Opts::new("udp_sent_bytes", "Number of bytes sent over UDP"), &["server", "type"]).unwrap();

    // UDP received
    static ref UDP_RECEIVED_COUNT: IntCounterVec = IntCounterVec::new(Opts::new("udp_received_count", "Number of UDP datagrams received"), &["server", "type"]).unwrap();
    static ref UDP_RECEIVED_BYTES: IntCounterVec = IntCounterVec::new(Opts::new("udp_received_bytes", "Number of bytes received over UDP"), &["server", "type"]).unwrap();
}

pub fn start(bind: String) {
    debug!("Registering Prometheus metrics");
    REGISTRY.register(Box::new(UDP_SENT_COUNT.clone())).unwrap();
    REGISTRY.register(Box::new(UDP_SENT_BYTES.clone())).unwrap();
    REGISTRY
        .register(Box::new(UDP_RECEIVED_COUNT.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(UDP_RECEIVED_BYTES.clone()))
        .unwrap();

    info!("Starting Prometheus metrics server, bind: {}", bind);
    let listener = TcpListener::bind(bind).expect("bind metrics server error");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_request(stream));
            }
            Err(err) => {
                error!("Unable to connect, error: {}", err);
            }
        }
    }
}

pub fn incr_udp_sent_count(server: &str, typ: &str) {
    UDP_SENT_COUNT.with_label_values(&[server, typ]).inc();
}

pub fn incr_udp_sent_bytes(server: &str, typ: &str, count: usize) {
    UDP_SENT_BYTES
        .with_label_values(&[server, typ])
        .inc_by(count as u64);
}

pub fn incr_udp_received_count(server: &str, typ: &str) {
    UDP_RECEIVED_COUNT.with_label_values(&[server, typ]).inc();
}

pub fn incr_udp_received_bytes(server: &str, typ: &str, count: usize) {
    UDP_RECEIVED_BYTES
        .with_label_values(&[server, typ])
        .inc_by(count as u64);
}

fn handle_request(stream: TcpStream) {
    handle_read(&stream);
    handle_write(stream);
}

fn handle_read(mut stream: &TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
}

fn handle_write(mut stream: TcpStream) {
    let encoder = prometheus::TextEncoder::new();
    if let Err(err) =
        stream.write(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n")
    {
        error!("Write http header error: {}", err);
        return;
    };

    let mut buffer = Vec::new();
    if let Err(err) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        error!("Encode Prometheus metrics error: {}", err);
        return;
    }

    if let Err(err) = stream.write(&buffer) {
        error!("Write metrics error: {}", err);
        return;
    };
}
