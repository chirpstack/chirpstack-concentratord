#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate simple_logger;

use std::str::FromStr;

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
                .help("Path to configuration file")
                .takes_value(true),
        )
        .get_matches();

    let config_filename = matches
        .value_of("config")
        .unwrap_or("chirpstack-concentratord-sx1302.toml");
    let config = config::get(config_filename);

    simple_logger::init_with_level(log::Level::from_str(&config.concentratord.log_level).unwrap())
        .unwrap();

    cmd::root::run(&config).unwrap();
}
