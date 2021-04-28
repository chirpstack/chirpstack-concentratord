#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate simple_logger;
extern crate syslog;

use std::process;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

use clap::{App, Arg};
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

fn main() {
    let matches = App::new("concentratord")
        .version(config::VERSION)
        .author("Orne Brocaar <info@brocaar.com>")
        .about("LoRa concentrator HAL daemon (2.4GHz)")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .multiple(true)
                .number_of_values(1)
                .help("Path to configuration file")
                .takes_value(true),
        )
        .subcommand(App::new("configfile").about("Print the configuration template"))
        .get_matches();

    let config_files = matches.values_of_lossy("config").unwrap_or(vec![]);
    let mut config = config::get(config_files);

    if let Some(_) = matches.subcommand_matches("configfile") {
        cmd::configfile::run(&config);
        process::exit(0);
    }

    if config.concentratord.log_to_syslog {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: "chirpstack-concentratord-sx1301".into(),
            pid: process::id() as i32,
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
            .with_level(
                log::Level::from_str(&config.concentratord.log_level)
                    .unwrap()
                    .to_level_filter(),
            )
            .init()
            .unwrap();
    }

    let mut signals = Signals::new(&[SIGINT]).expect("error registering channels");
    let (stop_send, stop_receive) = channel();
    let stop_receive = Arc::new(stop_receive);

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
    if config.gateway.model_config.reset_pin.is_some() {
        reset::setup_pins(config.gateway.model_config.reset_pin.unwrap(), None)
            .expect("setup reset pin error");
    }

    loop {
        match cmd::root::run(&config, stop_send.clone(), stop_receive.clone()).unwrap() {
            Signal::Stop => process::exit(0),
            Signal::Configuration(new_config) => {
                handler::config::update_configuration(&mut config, &new_config)
                    .expect("update configuration failed")
            }
        }
    }
}
