use libloragw_sx1302::hal;

use super::super::super::super::config;
use super::super::{ComType, Configuration, Gps, RadioConfig};

// source:
// https://github.com/RAKWireless/rak_common_for_gateway/blob/45c93c07f7/lora/rak2287/global_conf_uart/global_conf.eu_433.json
pub fn new(conf: &config::Configuration) -> Configuration {
    let gps = conf.gateway.model_flags.contains(&"GNSS".to_string());
    let usb = conf.gateway.model_flags.contains(&"USB".to_string());

    Configuration {
        radio_count: 2,
        clock_source: 0,
        full_duplex: false,
        lora_multi_sf_bandwidth: 125000,
        radio_config: vec![
            RadioConfig {
                enable: true,
                radio_type: hal::RadioType::SX1250,
                single_input_mode: true,
                rssi_offset: -207.0,
                rssi_temp_compensation: hal::RssiTempCompensationConfig {
                    coeff_a: 0.0,
                    coeff_b: 0.0,
                    coeff_c: 20.41,
                    coeff_d: 2162.56,
                    coeff_e: 0.0,
                },
                tx_enable: true,
                tx_freq_min: 433050000,
                tx_freq_max: 434900000,
                tx_gain_table: vec![
                    // 0
                    hal::TxGainConfig {
                        rf_power: -6,
                        pa_gain: 0,
                        pwr_idx: 0,
                        ..Default::default()
                    },
                    // 1
                    hal::TxGainConfig {
                        rf_power: -3,
                        pa_gain: 0,
                        pwr_idx: 1,
                        ..Default::default()
                    },
                    // 2
                    hal::TxGainConfig {
                        rf_power: 0,
                        pa_gain: 0,
                        pwr_idx: 2,
                        ..Default::default()
                    },
                    // 3
                    hal::TxGainConfig {
                        rf_power: 3,
                        pa_gain: 1,
                        pwr_idx: 3,
                        ..Default::default()
                    },
                    // 4
                    hal::TxGainConfig {
                        rf_power: 6,
                        pa_gain: 1,
                        pwr_idx: 4,
                        ..Default::default()
                    },
                    // 5
                    hal::TxGainConfig {
                        rf_power: 10,
                        pa_gain: 1,
                        pwr_idx: 5,
                        ..Default::default()
                    },
                    // 6
                    hal::TxGainConfig {
                        rf_power: 11,
                        pa_gain: 1,
                        pwr_idx: 6,
                        ..Default::default()
                    },
                    // 7
                    hal::TxGainConfig {
                        rf_power: 12,
                        pa_gain: 1,
                        pwr_idx: 7,
                        ..Default::default()
                    },
                    // 8
                    hal::TxGainConfig {
                        rf_power: 13,
                        pa_gain: 1,
                        pwr_idx: 8,
                        ..Default::default()
                    },
                    // 9
                    hal::TxGainConfig {
                        rf_power: 14,
                        pa_gain: 1,
                        pwr_idx: 9,
                        ..Default::default()
                    },
                    // 10
                    hal::TxGainConfig {
                        rf_power: 16,
                        pa_gain: 1,
                        pwr_idx: 10,
                        ..Default::default()
                    },
                    // 11
                    hal::TxGainConfig {
                        rf_power: 20,
                        pa_gain: 1,
                        pwr_idx: 11,
                        ..Default::default()
                    },
                    // 12
                    hal::TxGainConfig {
                        rf_power: 23,
                        pa_gain: 1,
                        pwr_idx: 12,
                        ..Default::default()
                    },
                    // 13
                    hal::TxGainConfig {
                        rf_power: 25,
                        pa_gain: 1,
                        pwr_idx: 13,
                        ..Default::default()
                    },
                    // 14
                    hal::TxGainConfig {
                        rf_power: 26,
                        pa_gain: 1,
                        pwr_idx: 14,
                        ..Default::default()
                    },
                    // 15
                    hal::TxGainConfig {
                        rf_power: 27,
                        pa_gain: 1,
                        pwr_idx: 15,
                        ..Default::default()
                    },
                ],
            },
            RadioConfig {
                enable: true,
                radio_type: hal::RadioType::SX1250,
                single_input_mode: true,
                rssi_offset: -207.0,
                rssi_temp_compensation: hal::RssiTempCompensationConfig {
                    coeff_a: 0.0,
                    coeff_b: 0.0,
                    coeff_c: 20.41,
                    coeff_d: 2162.56,
                    coeff_e: 0.0,
                },
                tx_enable: false,
                tx_freq_min: 0,
                tx_freq_max: 0,
                tx_gain_table: vec![],
            },
        ],
        gps: match gps {
            true => Gps::TtyPath("/dev/ttyAMA0".to_string()),
            false => Gps::None,
        },
        com_type: match usb {
            true => ComType::Usb,
            false => ComType::Spi,
        },
        com_path: match usb {
            true => "/dev/ttyACM0".to_string(),
            false => "/dev/spidev0.0".to_string(),
        },
        sx1302_reset_pin: match conf.gateway.sx1302_reset_pin {
            0 => Some(("/dev/gpiochip0".to_string(), 17)),
            _ => Some(("/dev/gpiochip0".to_string(), conf.gateway.sx1302_reset_pin)),
        },
        sx1302_power_en_pin: match conf.gateway.sx1302_power_en_pin {
            0 => None,
            _ => Some((
                "/dev/gpiochip0".to_string(),
                conf.gateway.sx1302_power_en_pin,
            )),
        },
        ..Default::default()
    }
}
