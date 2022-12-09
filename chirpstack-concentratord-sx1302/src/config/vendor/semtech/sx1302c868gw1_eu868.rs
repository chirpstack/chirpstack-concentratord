use libloragw_sx1302::hal;

use super::super::super::super::config;
use super::super::{ComType, Configuration, Gps, RadioConfig};

// source: https://github.com/Lora-net/sx1302_hal/blob/master/packet_forwarder/global_conf.json.sx1250.EU868
pub fn new(conf: &config::Configuration) -> Configuration {
    let gps = conf.gateway.model_flags.contains(&"GNSS".to_string());

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
                rssi_offset: -215.4,
                rssi_temp_compensation: hal::RssiTempCompensationConfig {
                    coeff_a: 0.0,
                    coeff_b: 0.0,
                    coeff_c: 20.41,
                    coeff_d: 2162.56,
                    coeff_e: 0.0,
                },
                tx_enable: true,
                tx_freq_min: 863000000,
                tx_freq_max: 870000000,
                tx_gain_table: vec![
                    // 0
                    hal::TxGainConfig {
                        rf_power: 12,
                        pa_gain: 0,
                        pwr_idx: 15,
                        ..Default::default()
                    },
                    // 1
                    hal::TxGainConfig {
                        rf_power: 13,
                        pa_gain: 0,
                        pwr_idx: 16,
                        ..Default::default()
                    },
                    // 2
                    hal::TxGainConfig {
                        rf_power: 14,
                        pa_gain: 0,
                        pwr_idx: 17,
                        ..Default::default()
                    },
                    // 3
                    hal::TxGainConfig {
                        rf_power: 15,
                        pa_gain: 0,
                        pwr_idx: 19,
                        ..Default::default()
                    },
                    // 4
                    hal::TxGainConfig {
                        rf_power: 16,
                        pa_gain: 0,
                        pwr_idx: 20,
                        ..Default::default()
                    },
                    // 5
                    hal::TxGainConfig {
                        rf_power: 17,
                        pa_gain: 0,
                        pwr_idx: 22,
                        ..Default::default()
                    },
                    // 6
                    hal::TxGainConfig {
                        rf_power: 18,
                        pa_gain: 1,
                        pwr_idx: 1,
                        ..Default::default()
                    },
                    // 7
                    hal::TxGainConfig {
                        rf_power: 19,
                        pa_gain: 1,
                        pwr_idx: 2,
                        ..Default::default()
                    },
                    // 8
                    hal::TxGainConfig {
                        rf_power: 20,
                        pa_gain: 1,
                        pwr_idx: 3,
                        ..Default::default()
                    },
                    // 9
                    hal::TxGainConfig {
                        rf_power: 21,
                        pa_gain: 1,
                        pwr_idx: 4,
                        ..Default::default()
                    },
                    // 10
                    hal::TxGainConfig {
                        rf_power: 22,
                        pa_gain: 1,
                        pwr_idx: 5,
                        ..Default::default()
                    },
                    // 11
                    hal::TxGainConfig {
                        rf_power: 23,
                        pa_gain: 1,
                        pwr_idx: 6,
                        ..Default::default()
                    },
                    // 12
                    hal::TxGainConfig {
                        rf_power: 24,
                        pa_gain: 1,
                        pwr_idx: 7,
                        ..Default::default()
                    },
                    // 13
                    hal::TxGainConfig {
                        rf_power: 25,
                        pa_gain: 1,
                        pwr_idx: 9,
                        ..Default::default()
                    },
                    // 14
                    hal::TxGainConfig {
                        rf_power: 26,
                        pa_gain: 1,
                        pwr_idx: 11,
                        ..Default::default()
                    },
                    // 15
                    hal::TxGainConfig {
                        rf_power: 27,
                        pa_gain: 1,
                        pwr_idx: 14,
                        ..Default::default()
                    },
                ],
            },
            RadioConfig {
                enable: true,
                radio_type: hal::RadioType::SX1250,
                single_input_mode: false,
                rssi_offset: -215.4,
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
        com_type: ComType::SPI,
        com_path: "/dev/spidev0.0".to_string(),
        sx1302_reset_pin: match conf.gateway.sx1302_reset_pin {
            0 => Some(("/dev/gpiochip0".to_string(), 23)),
            _ => Some(("/dev/gpiochip0".to_string(), conf.gateway.sx1302_reset_pin)),
        },
        sx1302_power_en_pin: match conf.gateway.sx1302_power_en_pin {
            0 => Some(("/dev/gpiochip0".to_string(), 18)),
            _ => Some((
                "/dev/gpiochip0".to_string(),
                conf.gateway.sx1302_power_en_pin,
            )),
        },
        sx1261_reset_pin: None,
        ad5338r_reset_pin: None,
        reset_commands: None,
    }
}
