use anyhow::Result;
use libloragw_sx1302::hal;

use super::super::super::super::config::{self, Region};
use super::super::{ComType, Configuration, Gps, RadioConfig};

// source: https://github.com/Lora-net/sx1302_hal/blob/master/packet_forwarder/global_conf.json.sx1250.CN490
pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let region = conf.gateway.region.unwrap_or(Region::CN470);

    let (tx_freq_min, tx_freq_max) = match region {
        Region::CN470 => (470_000_000, 510_000_000),
        Region::EU433 => (433_050_000, 434_900_000),
        _ => return Err(anyhow!("Region is not supported: {}", region)),
    };

    let gps = conf.gateway.model_flags.contains(&"GNSS".to_string());

    Ok(Configuration {
        radio_count: 2,
        clock_source: 0,
        full_duplex: false,
        lora_multi_sf_bandwidth: 125000,
        radio_config: vec![
            RadioConfig {
                tx_freq_min,
                tx_freq_max,
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
                single_input_mode: false,
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
            true => Gps::TtyPath(
                conf.gateway
                    .gnss_dev_path
                    .clone()
                    .unwrap_or("/dev/ttyAMA0".to_string()),
            ),
            false => Gps::None,
        },
        com_type: ComType::Spi,
        com_path: conf
            .gateway
            .com_dev_path
            .clone()
            .unwrap_or("/dev/spidev0.0".to_string()),
        i2c_path: Some(
            conf.gateway
                .i2c_dev_path
                .clone()
                .unwrap_or("/dev/i2c-1".to_string()),
        ),
        i2c_temp_sensor_addr: Some(0x3b),
        sx1302_reset_pin: conf.gateway.get_sx1302_reset_pin("/dev/gpiochip0", 23),
        sx1302_power_en_pin: conf.gateway.get_sx1302_power_en_pin("/dev/gpiochip0", 18),
        ..Default::default()
    })
}
