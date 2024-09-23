use super::super::super::super::config;
use super::super::Configuration;

pub fn new(conf: &config::Configuration) -> Configuration {
    Configuration {
        tty_path: conf.gateway.get_com_dev_path("/dev/ttyACM0"),
        tx_min_max_freqs: vec![(2400000000, 2483500000)],
        reset_pin: None,
        boot0_pin: None,
    }
}
