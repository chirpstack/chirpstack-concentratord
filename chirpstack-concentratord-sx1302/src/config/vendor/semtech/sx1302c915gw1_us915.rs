use libloragw_sx1302::hal;

use super::super::super::super::config;
use super::super::{ComType, Configuration, RadioConfig};

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
                rssi_offset: -207.0,
                rssi_temp_compensation: hal::RssiTempCompensationConfig {
                    coeff_a: 0.0,
                    coeff_b: 0.0,
                    coeff_c: 20.41,
                    coeff_d: 2162.56,
                    coeff_e: 0.0,
                },
                tx_enable: true,
                tx_freq_min: 923000000,
                tx_freq_max: 928000000,
                tx_gain_table: vec![
                    // 0
                    hal::TxGainConfig {
                        rf_power: 12,
                        pa_gain: 0,
                        pwr_idx: 15,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 1
                    hal::TxGainConfig {
                        rf_power: 13,
                        pa_gain: 0,
                        pwr_idx: 16,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 2
                    hal::TxGainConfig {
                        rf_power: 14,
                        pa_gain: 0,
                        pwr_idx: 17,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 3
                    hal::TxGainConfig {
                        rf_power: 15,
                        pa_gain: 0,
                        pwr_idx: 19,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 4
                    hal::TxGainConfig {
                        rf_power: 16,
                        pa_gain: 0,
                        pwr_idx: 20,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 5
                    hal::TxGainConfig {
                        rf_power: 17,
                        pa_gain: 0,
                        pwr_idx: 22,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 6
                    hal::TxGainConfig {
                        rf_power: 18,
                        pa_gain: 1,
                        pwr_idx: 1,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 7
                    hal::TxGainConfig {
                        rf_power: 19,
                        pa_gain: 1,
                        pwr_idx: 2,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 8
                    hal::TxGainConfig {
                        rf_power: 20,
                        pa_gain: 1,
                        pwr_idx: 3,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 9
                    hal::TxGainConfig {
                        rf_power: 21,
                        pa_gain: 1,
                        pwr_idx: 4,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 10
                    hal::TxGainConfig {
                        rf_power: 22,
                        pa_gain: 1,
                        pwr_idx: 5,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 11
                    hal::TxGainConfig {
                        rf_power: 23,
                        pa_gain: 1,
                        pwr_idx: 6,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 12
                    hal::TxGainConfig {
                        rf_power: 24,
                        pa_gain: 1,
                        pwr_idx: 7,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 13
                    hal::TxGainConfig {
                        rf_power: 25,
                        pa_gain: 1,
                        pwr_idx: 9,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 14
                    hal::TxGainConfig {
                        rf_power: 26,
                        pa_gain: 1,
                        pwr_idx: 11,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
                    },
                    // 15
                    hal::TxGainConfig {
                        rf_power: 27,
                        pa_gain: 1,
                        pwr_idx: 14,
                        dig_gain: 0,
                        dac_gain: 0,
                        mix_gain: 5,
                        offset_i: 0,
                        offset_q: 0,
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
        gps_tty_path: match gps {
            true => Some("/dev/ttyAMA0".to_string()),
            false => None,
        },
        com_type: ComType::SPI,
        com_path: "/dev/spidev0.0".to_string(),
        reset_pin: match conf.gateway.reset_pin {
            0 => Some(23),
            _ => Some(conf.gateway.reset_pin),
        },
        power_en_pin: match conf.gateway.power_en_pin {
            0 => Some(18),
            _ => Some(conf.gateway.power_en_pin),
        },
    }
}
