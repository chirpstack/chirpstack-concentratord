use anyhow::Result;
use libloragw_sx1301::hal;

use super::super::super::super::config::{self, Region};
use super::super::{Configuration, Gps};

// source:
// https://github.com/Wifx/meta-wifx/blob/krogoth/recipes-connectivity/packet-forwarder/files/configs/global_conf_EU868_2dBi_indoor.json
// https://github.com/Wifx/meta-wifx/blob/krogoth/recipes-connectivity/packet-forwarder/files/configs/global_conf_EU868_4dBi_outdoor.json
pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let region = conf.gateway.region.unwrap_or(Region::EU868);

    let radio_min_max_tx_freq = match region {
        Region::EU868 => vec![(863000000, 870000000), (863000000, 870000000)],
        _ => return Err(anyhow!("Region is not supported: {}", region)),
    };

    Ok(Configuration {
        radio_min_max_tx_freq,
        radio_count: 2,
        clock_source: 1,
        radio_rssi_offset: vec![-164.0, -164.0],
        radio_tx_enabled: vec![true, false],
        radio_type: vec![hal::RadioType::SX1257, hal::RadioType::SX1257],
        radio_tx_notch_freq: vec![129000, 0],
        lora_multi_sf_bandwidth: 125000,
        tx_gain_table: if conf.gateway.antenna_gain == 2 {
            vec![
                // 0
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 10,
                    rf_power: -6,
                    dig_gain: 1,
                    dac_gain: 3,
                },
                // 1
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 9,
                    rf_power: -3,
                    dig_gain: 3,
                    dac_gain: 3,
                },
                // 2
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 15,
                    rf_power: 0,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 3
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 9,
                    rf_power: 3,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 4
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 8,
                    rf_power: 6,
                    dig_gain: 2,
                    dac_gain: 3,
                },
                // 5
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 10,
                    rf_power: 10,
                    dig_gain: 2,
                    dac_gain: 3,
                },
                // 6
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 9,
                    rf_power: 11,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 7
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 11,
                    rf_power: 12,
                    dig_gain: 2,
                    dac_gain: 3,
                },
                // 8
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 11,
                    rf_power: 13,
                    dig_gain: 1,
                    dac_gain: 3,
                },
                // 9
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 12,
                    rf_power: 14,
                    dig_gain: 2,
                    dac_gain: 3,
                },
                // 10
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 12,
                    rf_power: 16,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 11
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 12,
                    rf_power: 20,
                    dig_gain: 3,
                    dac_gain: 3,
                },
                // 12
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 11,
                    rf_power: 23,
                    dig_gain: 1,
                    dac_gain: 3,
                },
                // 13
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 12,
                    rf_power: 25,
                    dig_gain: 1,
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
                    mix_gain: 13,
                    rf_power: 27,
                    dig_gain: 0,
                    dac_gain: 3,
                },
            ]
        } else if conf.gateway.antenna_gain == 4 {
            vec![
                // 0
                hal::TxGainConfig {
                    pa_gain: 0,
                    mix_gain: 9,
                    rf_power: -6,
                    dig_gain: 1,
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
                    pa_gain: 1,
                    mix_gain: 8,
                    rf_power: 0,
                    dig_gain: 2,
                    dac_gain: 3,
                },
                // 3
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 11,
                    rf_power: 3,
                    dig_gain: 3,
                    dac_gain: 3,
                },
                // 4
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 13,
                    rf_power: 6,
                    dig_gain: 3,
                    dac_gain: 3,
                },
                // 5
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 13,
                    rf_power: 10,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 6
                hal::TxGainConfig {
                    pa_gain: 1,
                    mix_gain: 15,
                    rf_power: 11,
                    dig_gain: 1,
                    dac_gain: 3,
                },
                // 7
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 10,
                    rf_power: 12,
                    dig_gain: 2,
                    dac_gain: 3,
                },
                // 8
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 10,
                    rf_power: 13,
                    dig_gain: 1,
                    dac_gain: 3,
                },
                // 9
                hal::TxGainConfig {
                    pa_gain: 2,
                    mix_gain: 11,
                    rf_power: 14,
                    dig_gain: 2,
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
                    pa_gain: 2,
                    mix_gain: 15,
                    rf_power: 20,
                    dig_gain: 2,
                    dac_gain: 3,
                },
                // 12
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 10,
                    rf_power: 23,
                    dig_gain: 1,
                    dac_gain: 3,
                },
                // 13
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 11,
                    rf_power: 25,
                    dig_gain: 1,
                    dac_gain: 3,
                },
                // 14
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 11,
                    rf_power: 26,
                    dig_gain: 0,
                    dac_gain: 3,
                },
                // 15
                hal::TxGainConfig {
                    pa_gain: 3,
                    mix_gain: 15,
                    rf_power: 27,
                    dig_gain: 3,
                    dac_gain: 3,
                },
            ]
        } else {
            panic!("Invalid antenna_gain: {}", conf.gateway.antenna_gain);
        },
        gps: Gps::None,
        spidev_path: "/dev/spidev0.0".to_string(),
        reset_pin: conf.gateway.get_sx1301_reset_pin("/dev/gpiochip0", 1),
    })
}
