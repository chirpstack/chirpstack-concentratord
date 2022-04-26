use super::super::super::super::config;
use super::super::Configuration;

pub fn new(conf: &config::Configuration) -> Configuration {
    let mut c = super::super::rak::rak2247_in865::new(conf);
    c.reset_pin = match conf.gateway.reset_pin {
        0 => Some((0, 22)),
        _ => Some((0, conf.gateway.reset_pin)),
    };
    c
}
