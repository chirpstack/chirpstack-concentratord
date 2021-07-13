use super::super::super::super::config;
use super::super::Configuration;

pub fn new(_conf: &config::Configuration) -> Configuration {
    Configuration {
        tty_path: "/dev/ttyACM0".to_string(),
        min_max_tx_freq: (2400000000, 2483500000),
        reset_pin: Some(32),
        boot0_pin: Some(18),
    }
}
