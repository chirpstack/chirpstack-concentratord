use log::warn;

use super::super::super::super::config;
use super::super::Configuration;

pub fn new(conf: &config::Configuration) -> Configuration {
    warn!("Deprecation warning: please use model pi_supply_lora_gateway_hat and specify region");

    let mut c = super::super::rak::rak2247_ru864::new(conf);
    c.reset_pin = match conf.gateway.reset_pin {
        0 => Some(("/dev/gpiochip0".to_string(), 22)),
        _ => Some(("/dev/gpiochip0".to_string(), conf.gateway.reset_pin)),
    };
    c
}
