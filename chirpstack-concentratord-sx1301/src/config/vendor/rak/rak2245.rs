use anyhow::Result;
use libloragw_sx1301::hal;

use super::super::super::super::config::{self, Region};
use super::super::{Configuration, Gps};

// source: https://github.com/RAKWireless/rak_common_for_gateway/blob/099555865a42238f125c68ded5233a985747c40d/lora/rak2245/global_conf/
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

    let radio_min_max_tx_freq = match region {
        Region::AS923 | Region::AS923_2 | Region::AS923_3 | Region::AS923_4 => {
            vec![(915000000, 928000000), (915000000, 928000000)]
        }
        Region::AU915 => vec![(915000000, 928000000), (915000000, 928000000)],
        Region::CN470 => vec![(470000000, 510000000), (470000000, 510000000)],
        Region::EU433 => vec![(433050000, 434900000), (433050000, 434900000)],
        Region::EU868 => vec![(863000000, 870000000), (863000000, 870000000)],
        Region::IN865 => vec![(865000000, 867000000), (865000000, 867000000)],
        Region::KR920 => vec![(920900000, 923300000), (920900000, 923300000)],
        Region::RU864 => vec![(863000000, 870000000), (863000000, 870000000)],
        Region::US915 => vec![(902000000, 928000000), (902000000, 928000000)],
        _ => return Err(anyhow!("Region is not supported: {}", region)),
    };

    let tx_gain_table = match region {
        Region::AS923
        | Region::AS923_2
        | Region::AS923_3
        | Region::AS923_4
        | Region::AU915
        | Region::KR920
        | Region::US915 => {
            vec![
                // 0
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 8,
                    rf_power: -6,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 1
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 11,
                    rf_power: -3,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 2
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 14,
                    rf_power: 0,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 3
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 8,
                    rf_power: 3,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 4
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 10,
                    rf_power: 6,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 5
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 14,
                    rf_power: 10,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 6
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 8,
                    rf_power: 11,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 7
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 8,
                    rf_power: 12,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 8
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 9,
                    rf_power: 13,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 9
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 10,
                    rf_power: 14,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 10
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 11,
                    rf_power: 16,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 11
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 8,
                    rf_power: 20,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 12
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 9,
                    rf_power: 23,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 13
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 11,
                    rf_power: 25,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 14
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 13,
                    rf_power: 26,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 15
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 14,
                    rf_power: 27,
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
                dac_gain: 3,
            },
            // 1
            hal::TxGainConfig {
                pa_gain: 0,
                mix_gain: 10,
                rf_power: -3,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 2
            hal::TxGainConfig {
                pa_gain: 0,
                mix_gain: 12,
                rf_power: 0,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 3
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 8,
                rf_power: 3,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 4
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 10,
                rf_power: 6,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 5
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 12,
                rf_power: 10,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 6
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 13,
                rf_power: 11,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 7
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 9,
                rf_power: 12,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 8
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 15,
                rf_power: 13,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 9
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 10,
                rf_power: 14,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 10
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 11,
                rf_power: 16,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 11
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 9,
                rf_power: 20,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 12
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 10,
                rf_power: 23,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 13
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 11,
                rf_power: 25,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 14
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 12,
                rf_power: 26,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 15
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 14,
                rf_power: 27,
                dig_gain: 0,
                dac_gain: 3,
            },
        ],
        Region::EU868 | Region::IN865 | Region::RU864 => vec![
            // 0
            hal::TxGainConfig {
                pa_gain: 0,
                mix_gain: 11,
                rf_power: -6,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 1
            hal::TxGainConfig {
                pa_gain: 0,
                mix_gain: 14,
                rf_power: -3,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 2
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 9,
                rf_power: 0,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 3
            hal::TxGainConfig {
                pa_gain: 1,
                mix_gain: 11,
                rf_power: 3,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 4
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 8,
                rf_power: 6,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 5
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 10,
                rf_power: 10,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 6
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 11,
                rf_power: 11,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 7
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 11,
                rf_power: 12,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 8
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 12,
                rf_power: 13,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 9
            hal::TxGainConfig {
                pa_gain: 2,
                mix_gain: 12,
                rf_power: 14,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 10
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 8,
                rf_power: 16,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 11
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 10,
                rf_power: 20,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 12
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 12,
                rf_power: 23,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 13
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 13,
                rf_power: 25,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 14
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 14,
                rf_power: 26,
                dig_gain: 0,
                dac_gain: 3,
            },
            // 15
            hal::TxGainConfig {
                pa_gain: 3,
                mix_gain: 15,
                rf_power: 27,
                dig_gain: 0,
                dac_gain: 3,
            },
        ],
        _ => return Err(anyhow!("Region is not supported: {}", region)),
    };

    let gps = conf.gateway.model_flags.contains(&"GNSS".to_string());
    let enforce_duty_cycle = conf.gateway.model_flags.contains(&"ENFORCE_DC".to_string());

    Ok(Configuration {
        radio_rssi_offset,
        radio_type,
        radio_min_max_tx_freq,
        tx_gain_table,
        enforce_duty_cycle,
        radio_count: 2,
        clock_source: 1,
        radio_tx_enabled: vec![true, false],
        radio_tx_notch_freq: vec![0, 0],
        lora_multi_sf_bandwidth: 125000,
        gps: match gps {
            true => Gps::TtyPath(
                conf.gateway
                    .gnss_dev_path
                    .clone()
                    .unwrap_or("/dev/ttyAMA0".to_string()),
            ),
            false => Gps::None,
        },
        spidev_path: conf
            .gateway
            .com_dev_path
            .clone()
            .unwrap_or("/dev/spidev0.0".to_string()),
        reset_pin: conf.gateway.get_sx1301_reset_pin("/dev/gpiochip0", 17),
    })
}
