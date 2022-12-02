use super::super::super::super::config;
use super::super::Configuration;

pub fn new(conf: &config::Configuration) -> Configuration {
    let mut c = super::super::rak::rak2247_au915::new(conf);
    c.reset_pin = match conf.gateway.reset_pin {
        0 => Some(("/dev/gpiochip0".to_string(), 22)),
        _ => Some(("/dev/gpiochip0".to_string(), conf.gateway.reset_pin)),
    };
    c
}
