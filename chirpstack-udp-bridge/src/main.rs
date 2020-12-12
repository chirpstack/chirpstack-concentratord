#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::str::FromStr;
use std::thread;

use clap::{App, Arg};

use libconcentratord::logging;

mod config;
mod forwarder;
mod helpers;
mod metrics;
mod signals;
mod structs;

fn main() {
    let matches = App::new("chirpstack-udp-bridge")
        .version(config::VERSION)
        .author("Orne Brocaar <info@brocaar.com>")
        .about("ChirpStack UDP bridge for Concentratord compatible with the Semtech UDP protocol")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Path to configuration file")
                .takes_value(true),
        )
        .get_matches();

    let config_file = matches.value_of_lossy("config").unwrap();
    let config = config::Configuration::get(&config_file).expect("read configuration error");
    let log_level =
        log::Level::from_str(&config.udp_bridge.log_level).expect("parse log_level error");

    logging::setup(
        &"chirpstack-udp-bridge",
        log_level,
        config.udp_bridge.log_to_syslog,
    )
    .expect("setup logger error");

    info!(
        "Starting UDP Forwarder (version: {}, docs: {})",
        config::VERSION,
        "https://www.chirpstack.io/concentratord/"
    );

    // read gateway id.
    let gateway_id =
        helpers::get_gateway_id(&config.concentratord.command_url).expect("get gateway_id error");

    info!(
        "Received gateway ID from Concentratord, gateway_id: {}",
        hex::encode(&gateway_id)
    );

    // setup threads
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    // servers
    for server in config.udp_bridge.servers {
        threads.push(thread::spawn({
            let gateway_id = gateway_id.clone();
            let event_url = config.concentratord.event_url.clone();
            let command_url = config.concentratord.command_url.clone();

            move || forwarder::start(&server, event_url, command_url, gateway_id)
        }));
    }

    // metrics
    if config.udp_bridge.metrics_bind != "" {
        threads.push(thread::spawn({
            let bind = config.udp_bridge.metrics_bind.clone();
            move || metrics::start(bind)
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}
