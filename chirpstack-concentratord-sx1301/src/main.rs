#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
extern crate simple_logger;
extern crate syslog;

use std::process;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;

use clap::{Parser, Subcommand};
use signal_hook::consts::signal::SIGINT;
use signal_hook::iterator::Signals;
use simple_logger::SimpleLogger;
use syslog::{BasicLogger, Facility, Formatter3164};

use libconcentratord::reset;
use libconcentratord::signals::Signal;

mod cmd;
mod concentrator;
mod config;
mod handler;
mod wrapper;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the configuration template
    Configfile {},
}

fn main() {
    let cli = Cli::parse();
    let mut config = config::get(cli.config.clone());

    if let Some(Commands::Configfile {}) = &cli.command {
        cmd::configfile::run(&config);
        process::exit(0);
    }

    if config.concentratord.log_to_syslog {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: "chirpstack-concentratord-sx1301".into(),
            pid: process::id(),
        };
        let logger = syslog::unix(formatter).expect("could not connect to syslog");
        log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
            .map(|()| {
                log::set_max_level(
                    log::Level::from_str(&config.concentratord.log_level)
                        .unwrap()
                        .to_level_filter(),
                )
            })
            .unwrap();
    } else {
        SimpleLogger::new()
            .with_utc_timestamps()
            .with_level(
                log::Level::from_str(&config.concentratord.log_level)
                    .unwrap()
                    .to_level_filter(),
            )
            .init()
            .unwrap();
    }

    let mut signals = Signals::new([SIGINT]).expect("error registering channels");
    let (stop_send, stop_receive) = channel();
    let stop_receive = Rc::new(stop_receive);

    thread::spawn({
        let stop_send = stop_send.clone();

        move || {
            let mut signal_iter = signals.forever();
            let _ = signal_iter.next();
            warn!("Received stop signal, stopping Concentratord");
            stop_send.send(Signal::Stop).unwrap();
            let _ = signal_iter.next();
            warn!("Received stop signal, terminating Concentratord immediately");
            process::exit(0);
        }
    });

    // configure concentrator reset pin
    reset::setup_pins(reset::Configuration {
        sx130x_reset: config.gateway.model_config.reset_pin.clone(),
        ..Default::default()
    })
    .expect("setup reset pin error");

    loop {
        match cmd::root::run(&config, stop_send.clone(), stop_receive.clone()).unwrap() {
            Signal::Stop => process::exit(0),
            Signal::Configuration(new_config) => {
                handler::config::update_configuration(&mut config, &new_config)
                    .expect("update configuration failed");
            }
        }
    }
}
