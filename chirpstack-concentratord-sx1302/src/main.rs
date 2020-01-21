#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate simple_logger;
extern crate syslog;

use std::process;
use std::str::FromStr;

use syslog::{BasicLogger, Facility, Formatter3164};

mod cmd;
mod concentrator;
mod config;
mod handler;
mod wrapper;

use clap::{App, Arg};

fn main() {
    let matches = App::new("chirpstack-concentratord-sx1302")
        .version(config::VERSION)
        .author("Orne Brocaar <info@brocaar.com>")
        .about("LoRa concentrator HAL daemon for SX1302")
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
        .get_matches();

    let config_files = matches
        .values_of_lossy("config")
        .unwrap_or(vec!["chirpstack-concentratord-sx1302.toml".to_string()]);
    let config = config::get(config_files);

    if config.concentratord.log_to_syslog {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: "chirpstack-concentratord-sx1302".into(),
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
        simple_logger::init_with_level(
            log::Level::from_str(&config.concentratord.log_level).unwrap(),
        )
        .unwrap();
    }

    cmd::root::run(&config).unwrap();
}
