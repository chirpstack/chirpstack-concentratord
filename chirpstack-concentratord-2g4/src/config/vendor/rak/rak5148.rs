use super::super::super::super::config;
use super::super::Configuration;
use libconcentratord::region;

pub fn new(conf: &config::Configuration) -> Configuration {
    Configuration {
        tty_path: conf.gateway.get_com_dev_path("/dev/ttyACM0"),
        tx_min_max_freqs: region::ism2400::TX_MIN_MAX_FREQS.to_vec(),
        reset_pin: None,
        boot0_pin: None,
    }
}
