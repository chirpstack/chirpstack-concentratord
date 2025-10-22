use anyhow::Result;
use libloragw_sx1302::hal;

use super::super::super::super::config::{self, Region};
use super::super::{ComType, Configuration, RadioConfig};
use libconcentratord::{gnss, region};

// source:
// wget https://www.dragino.com/downloads/downloads/LoRa_Gateway/PG1302/software/draginofwd-32bit.deb
pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let region = conf
        .gateway
        .region
        .ok_or_else(|| anyhow!("You must specify a region"))?;

    let tx_min_max_freqs = match region {
        Region::EU868 => region::eu868::TX_MIN_MAX_FREQS.to_vec(),
        Region::US915 => region::us915::TX_MIN_MAX_FREQS.to_vec(),
        _ => return Err(anyhow!("Unsupported region: {}", region)),
    };

    let tx_gain_table = match region {
        Region::EU868 | Region::US915 => vec![
            // 0
            hal::TxGainConfig {
                rf_power: 12,
                pa_gain: 0,
                pwr_idx: 16,
                ..Default::default()
            },
            // 1
            hal::TxGainConfig {
                rf_power: 13,
                pa_gain: 0,
                pwr_idx: 17,
                ..Default::default()
            },
            // 2
            hal::TxGainConfig {
                rf_power: 14,
                pa_gain: 0,
                pwr_idx: 18,
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
                pwr_idx: 21,
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
                pwr_idx: 3,
                ..Default::default()
            },
            // 7
            hal::TxGainConfig {
                rf_power: 19,
                pa_gain: 1,
                pwr_idx: 4,
                ..Default::default()
            },
            // 8
            hal::TxGainConfig {
                rf_power: 20,
                pa_gain: 1,
                pwr_idx: 5,
                ..Default::default()
            },
            // 9
            hal::TxGainConfig {
                rf_power: 21,
                pa_gain: 1,
                pwr_idx: 6,
                ..Default::default()
            },
            // 10
            hal::TxGainConfig {
                rf_power: 22,
                pa_gain: 1,
                pwr_idx: 7,
                ..Default::default()
            },
            // 11
            hal::TxGainConfig {
                rf_power: 23,
                pa_gain: 1,
                pwr_idx: 8,
                ..Default::default()
            },
            // 12
            hal::TxGainConfig {
                rf_power: 24,
                pa_gain: 1,
                pwr_idx: 9,
                ..Default::default()
            },
            // 13
            hal::TxGainConfig {
                rf_power: 25,
                pa_gain: 1,
                pwr_idx: 11,
                ..Default::default()
            },
            // 14
            hal::TxGainConfig {
                rf_power: 26,
                pa_gain: 1,
                pwr_idx: 13,
                ..Default::default()
            },
            // 15
            hal::TxGainConfig {
                rf_power: 27,
                pa_gain: 1,
                pwr_idx: 17,
                ..Default::default()
            },
        ],
        _ => return Err(anyhow!("Unsupported region: {}", region)),
    };

    let enforce_duty_cycle = conf.gateway.model_flags.contains(&"ENFORCE_DC".to_string());

    Ok(Configuration {
        enforce_duty_cycle,
        radio_count: 2,
        clock_source: 0,
        full_duplex: false,
        lora_multi_sf_bandwidth: 125000,
        radio_config: vec![
            RadioConfig {
                tx_min_max_freqs,
                tx_gain_table,
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
        com_path: conf.gateway.get_com_dev_path("/dev/spidev0.0"),
        sx1302_reset_pin: conf.gateway.get_sx1302_reset_pin("/dev/gpiochip0", 23),
        ..Default::default()
    })
}
