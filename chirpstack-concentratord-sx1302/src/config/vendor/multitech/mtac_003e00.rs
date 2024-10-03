use anyhow::Result;
use libloragw_sx1302::hal;

use super::super::super::super::config::{self, Region};
use super::super::{ComType, Configuration, RadioConfig};
use libconcentratord::{gnss, region};

pub enum Port {
    AP1,
    AP2,
}

// source:
// mPower FW (/opt/lora/global_conf.json.MTAC_003_0_0.EU868)
pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let region = conf.gateway.region.unwrap_or(Region::EU868);

    let tx_min_max_freqs = match region {
        Region::EU868 => region::eu868::TX_MIN_MAX_FREQS.to_vec(),
        _ => return Err(anyhow!("Unsupported region: {}", region)),
    };

    let gps = conf.gateway.model_flags.contains(&"GNSS".to_string());
    let port = if conf.gateway.model_flags.contains(&"AP2".to_string()) {
        Port::AP2
    } else {
        Port::AP1
    };

    Ok(Configuration {
        radio_count: 2,
        clock_source: 0,
        full_duplex: false,
        lora_multi_sf_bandwidth: 125000,
        radio_config: vec![
            RadioConfig {
                tx_min_max_freqs,
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
                tx_gain_table: vec![
                    // 0
                    hal::TxGainConfig {
                        rf_power: 10,
                        pa_gain: 0,
                        pwr_idx: 12,
                        ..Default::default()
                    },
                    // 1
                    hal::TxGainConfig {
                        rf_power: 11,
                        pa_gain: 0,
                        pwr_idx: 13,
                        ..Default::default()
                    },
                    // 2
                    hal::TxGainConfig {
                        rf_power: 12,
                        pa_gain: 0,
                        pwr_idx: 14,
                        ..Default::default()
                    },
                    // 3
                    hal::TxGainConfig {
                        rf_power: 13,
                        pa_gain: 0,
                        pwr_idx: 15,
                        ..Default::default()
                    },
                    // 4
                    hal::TxGainConfig {
                        rf_power: 14,
                        pa_gain: 0,
                        pwr_idx: 16,
                        ..Default::default()
                    },
                    // 5
                    hal::TxGainConfig {
                        rf_power: 16,
                        pa_gain: 0,
                        pwr_idx: 17,
                        ..Default::default()
                    },
                    // 6
                    hal::TxGainConfig {
                        rf_power: 17,
                        pa_gain: 1,
                        pwr_idx: 0,
                        ..Default::default()
                    },
                    // 7
                    hal::TxGainConfig {
                        rf_power: 18,
                        pa_gain: 1,
                        pwr_idx: 1,
                        ..Default::default()
                    },
                    // 8
                    hal::TxGainConfig {
                        rf_power: 19,
                        pa_gain: 1,
                        pwr_idx: 2,
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
                        pwr_idx: 8,
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
                tx_min_max_freqs: vec![],
                tx_gain_table: vec![],
            },
        ],
        gps: match gps {
            true => conf
                .gateway
                .get_gnss_dev_path(&gnss::Device::new("gpsd://localhost:2947")),
            false => gnss::Device::None,
        },
        com_type: ComType::Spi,
        com_path: match port {
            Port::AP1 => conf.gateway.get_com_dev_path("/dev/spidev0.0"),
            Port::AP2 => conf.gateway.get_com_dev_path("/dev/spidev1.0"),
        },
        i2c_path: Some(conf.gateway.get_i2c_dev_path("/dev/i2c-1")),
        i2c_temp_sensor_addr: match port {
            Port::AP1 => Some(0x48),
            Port::AP2 => Some(0x49),
        },
        reset_commands: Some(match port {
            Port::AP1 => vec![
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap1/creset".to_string(),
                        "1".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap1/creset".to_string(),
                        "0".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap1/lbtreset".to_string(),
                        "0".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap1/lbtreset".to_string(),
                        "1".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap1/reset".to_string(),
                        "0".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap1/reset".to_string(),
                        "1".to_string(),
                    ],
                ),
            ],
            Port::AP2 => vec![
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap2/creset".to_string(),
                        "1".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap2/creset".to_string(),
                        "0".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap2/lbtreset".to_string(),
                        "0".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap2/lbtreset".to_string(),
                        "1".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap2/reset".to_string(),
                        "0".to_string(),
                    ],
                ),
                (
                    "mts-io-sysfs".to_string(),
                    vec![
                        "store".to_string(),
                        "ap2/reset".to_string(),
                        "1".to_string(),
                    ],
                ),
            ],
        }),
        ..Default::default()
    })
}
