use anyhow::Result;
use libconcentratord::{gnss, region};
use libloragw_sx1302::hal;

use super::super::super::super::config::{self, Region};
use super::super::{ComType, Configuration, RadioConfig};

// source:
// mPower FW (/opt/lora/global_conf.json.MTCAP3.US915)
pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let region = conf.gateway.region.unwrap_or(Region::US915);

    let tx_min_max_freqs = match region {
        Region::US915 => region::us915::TX_MIN_MAX_FREQS.to_vec(),
        _ => return Err(anyhow!("Unsupported region: {}", region)),
    };

    Ok(Configuration {
        radio_count: 2,
        clock_source: 0,
        full_duplex: false,
        lora_multi_sf_bandwidth: 125000,
        radio_config: vec![
            RadioConfig {
                tx_min_max_freqs,
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
                tx_gain_table: vec![
                    // 0
                    hal::TxGainConfig {
                        rf_power: 11,
                        pa_gain: 0,
                        pwr_idx: 14,
                        ..Default::default()
                    },
                    // 1
                    hal::TxGainConfig {
                        rf_power: 12,
                        pa_gain: 0,
                        pwr_idx: 15,
                        ..Default::default()
                    },
                    // 2
                    hal::TxGainConfig {
                        rf_power: 13,
                        pa_gain: 0,
                        pwr_idx: 16,
                        ..Default::default()
                    },
                    // 3
                    hal::TxGainConfig {
                        rf_power: 15,
                        pa_gain: 0,
                        pwr_idx: 17,
                        ..Default::default()
                    },
                    // 4
                    hal::TxGainConfig {
                        rf_power: 16,
                        pa_gain: 0,
                        pwr_idx: 18,
                        ..Default::default()
                    },
                    // 5
                    hal::TxGainConfig {
                        rf_power: 17,
                        pa_gain: 0,
                        pwr_idx: 19,
                        ..Default::default()
                    },
                    // 6
                    hal::TxGainConfig {
                        rf_power: 18,
                        pa_gain: 0,
                        pwr_idx: 20,
                        ..Default::default()
                    },
                    // 7
                    hal::TxGainConfig {
                        rf_power: 19,
                        pa_gain: 1,
                        pwr_idx: 3,
                        ..Default::default()
                    },
                    // 8
                    hal::TxGainConfig {
                        rf_power: 20,
                        pa_gain: 1,
                        pwr_idx: 4,
                        ..Default::default()
                    },
                    // 9
                    hal::TxGainConfig {
                        rf_power: 21,
                        pa_gain: 1,
                        pwr_idx: 5,
                        ..Default::default()
                    },
                    // 10
                    hal::TxGainConfig {
                        rf_power: 22,
                        pa_gain: 1,
                        pwr_idx: 6,
                        ..Default::default()
                    },
                    // 11
                    hal::TxGainConfig {
                        rf_power: 23,
                        pa_gain: 1,
                        pwr_idx: 7,
                        ..Default::default()
                    },
                    // 12
                    hal::TxGainConfig {
                        rf_power: 24,
                        pa_gain: 1,
                        pwr_idx: 8,
                        ..Default::default()
                    },
                    // 13
                    hal::TxGainConfig {
                        rf_power: 25,
                        pa_gain: 1,
                        pwr_idx: 10,
                        ..Default::default()
                    },
                    // 14
                    hal::TxGainConfig {
                        rf_power: 26,
                        pa_gain: 1,
                        pwr_idx: 12,
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
                tx_min_max_freqs: vec![],
                tx_gain_table: vec![],
            },
        ],
        gps: gnss::Device::None,
        com_type: ComType::Spi,
        com_path: conf.gateway.get_com_dev_path("/dev/spidev1.0"),
        reset_commands: Some(vec![
            (
                "mts-io-sysfs".to_string(),
                vec![
                    "store".to_string(),
                    "lora/reset".to_string(),
                    "0".to_string(),
                ],
            ),
            (
                "mts-io-sysfs".to_string(),
                vec![
                    "store".to_string(),
                    "lora/reset".to_string(),
                    "1".to_string(),
                ],
            ),
        ]),
        ..Default::default()
    })
}
