use anyhow::Result;
use libloragw_sx1301::hal;

use super::super::super::super::config::{self, Region};
use super::super::Configuration;
use libconcentratord::gnss;
use libconcentratord::region;

// source:
// https://github.com/RAKWireless/rak_common_for_gateway/blob/713ebf74f65beecdbc0304c7d880d05890f84315/lora/rak2246/global_conf/
pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let region = conf
        .gateway
        .region
        .ok_or_else(|| anyhow!("You must specify a region"))?;

    let radio_type = match region {
        Region::AS923
        | Region::AS923_2
        | Region::AS923_3
        | Region::AS923_4
        | Region::AU915
        | Region::EU868
        | Region::IN865
        | Region::KR920
        | Region::RU864
        | Region::US915 => {
            vec![hal::RadioType::SX1257, hal::RadioType::SX1257]
        }
        Region::CN470 | Region::EU433 => vec![hal::RadioType::SX1255, hal::RadioType::SX1255],
        _ => return Err(anyhow!("Region is not supported: {}", region)),
    };

    let radio_rssi_offset = match region {
        Region::AS923
        | Region::AS923_2
        | Region::AS923_3
        | Region::AS923_4
        | Region::AU915
        | Region::EU433
        | Region::EU868
        | Region::IN865
        | Region::KR920
        | Region::RU864
        | Region::US915 => {
            vec![-166.0, -166.0]
        }
        Region::CN470 => vec![-176.0, -176.0],
        _ => return Err(anyhow!("Region is not supported: {}", region)),
    };

    let tx_min_max_freqs = match region {
        Region::AS923 => region::as923::TX_MIN_MAX_FREQS.to_vec(),
        Region::AS923_2 => region::as923_2::TX_MIN_MAX_FREQS.to_vec(),
        Region::AS923_3 => region::as923_3::TX_MIN_MAX_FREQS.to_vec(),
        Region::AS923_4 => region::as923_4::TX_MIN_MAX_FREQS.to_vec(),
        Region::AU915 => region::au915::TX_MIN_MAX_FREQS.to_vec(),
        Region::CN470 => region::cn470::TX_MIN_MAX_FREQS.to_vec(),
        Region::EU433 => region::eu433::TX_MIN_MAX_FREQS.to_vec(),
        Region::EU868 => region::eu868::TX_MIN_MAX_FREQS.to_vec(),
        Region::IN865 => region::in865::TX_MIN_MAX_FREQS.to_vec(),
        Region::KR920 => region::kr920::TX_MIN_MAX_FREQS.to_vec(),
        Region::RU864 => region::ru864::TX_MIN_MAX_FREQS.to_vec(),
        Region::US915 => region::us915::TX_MIN_MAX_FREQS.to_vec(),
        _ => return Err(anyhow!("Region is not supported: {}", region)),
    };

    let tx_gain_table = match region {
        Region::AS923
        | Region::AS923_2
        | Region::AS923_3
        | Region::AS923_4
        | Region::AU915
        | Region::EU868
        | Region::IN865
        | Region::KR920
        | Region::RU864
        | Region::US915 => {
            vec![
                // 0
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 8,
                    rf_power: 13,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 1
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 9,
                    rf_power: 15,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 2
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 10,
                    rf_power: 17,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 3
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 11,
                    rf_power: 18,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 4
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 12,
                    rf_power: 19,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 5
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 13,
                    rf_power: 20,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 6
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 14,
                    rf_power: 21,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 7
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 15,
                    rf_power: 22,
                    dig_gain: 0,
                    dac_gain: 3,
                },
            ]
        }
        Region::CN470 | Region::EU433 => vec![
            // 0
            hal::TxGainConfig {
                pa_gain: 0,
                mix_gain: 8,
                rf_power: -6,
                dig_gain: 0,
                ..Default::default()
            },
            // 1
            hal::TxGainConfig {
                pa_gain: 0,
                mix_gain: 10,
                rf_power: -3,
                dig_gain: 0,
                ..Default::default()
            },
            // 2
            hal::TxGainConfig {
                pa_gain: 0,
                mix_gain: 12,
                rf_power: 0,
                dig_gain: 0,
                ..Default::default()
            },
            // 3
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 8,
                rf_power: 3,
                dig_gain: 0,
                ..Default::default()
            },
            // 4
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 10,
                rf_power: 6,
                dig_gain: 0,
                ..Default::default()
            },
            // 5
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 12,
                rf_power: 10,
                dig_gain: 0,
                ..Default::default()
            },
            // 6
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 13,
                rf_power: 11,
                dig_gain: 0,
                ..Default::default()
            },
            // 7
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 9,
                rf_power: 12,
                dig_gain: 0,
                ..Default::default()
            },
            // 8
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 15,
                rf_power: 13,
                dig_gain: 0,
                ..Default::default()
            },
            // 9
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 10,
                rf_power: 14,
                dig_gain: 0,
                ..Default::default()
            },
            // 10
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 11,
                rf_power: 16,
                dig_gain: 0,
                ..Default::default()
            },
            // 11
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 9,
                rf_power: 20,
                dig_gain: 0,
                ..Default::default()
            },
            // 12
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 10,
                rf_power: 23,
                dig_gain: 0,
                ..Default::default()
            },
            // 13
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 11,
                rf_power: 25,
                dig_gain: 0,
                ..Default::default()
            },
            // 14
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 12,
                rf_power: 26,
                dig_gain: 0,
                ..Default::default()
            },
            // 15
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 14,
                rf_power: 27,
                dig_gain: 0,
                ..Default::default()
            },
        ],
        _ => return Err(anyhow!("Region is not supported: {}", region)),
    };

    let gps = conf.gateway.model_flags.contains(&"GNSS".to_string());
    let enforce_duty_cycle = conf.gateway.model_flags.contains(&"ENFORCE_DC".to_string());

    Ok(Configuration {
        tx_gain_table,
        radio_type,
        radio_rssi_offset,
        tx_min_max_freqs,
        enforce_duty_cycle,
        radio_count: 2,
        clock_source: 1,
        radio_tx_enabled: vec![true, false],
        radio_tx_notch_freq: vec![0, 0],
        lora_multi_sf_bandwidth: 125000,
        gps: match gps {
            true => conf
                .gateway
                .get_gnss_dev_path(&gnss::Device::new("/dev/ttyAMA0")),
            false => gnss::Device::None,
        },
        spidev_path: conf.gateway.get_com_dev_path("/dev/spidev0.0"),
        reset_pin: conf.gateway.get_sx1301_reset_pin("/dev/gpiochip0", 17),
    })
}
