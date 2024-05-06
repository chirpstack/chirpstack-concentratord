use anyhow::Result;
use libloragw_sx1302::hal;

use super::super::super::super::config::{self, Region};
use super::super::{ComType, Configuration, RadioConfig};

// The Seeed wiki for the WM1302 points to the Semtech source:
// https://wiki.seeedstudio.com/WM1302_module/#step3-get-and-compile-sx1302-source-code
pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let region = conf
        .gateway
        .region
        .ok_or_else(|| anyhow!("You must specify a region"))?;

    let (tx_freq_min, tx_freq_max) = match region {
        Region::AS923 | Region::AS923_2 | Region::AS923_3 | Region::AS923_4 => {
            (915_000_000, 928_000_000)
        }
        Region::AU915 => (915_000_000, 928_000_000),
        Region::EU868 => (863_000_000, 870_000_000),
        Region::IN865 => (865_000_000, 867_000_000),
        Region::KR920 => (920_900_000, 923_300_000),
        Region::RU864 => (863_000_000, 870_000_000),
        Region::US915 => (923_000_000, 928_000_000),
        _ => return Err(anyhow!("Region not supported: {}", region)),
    };

    let rssi_offset = match region {
        Region::AS923
        | Region::AS923_2
        | Region::AS923_3
        | Region::AS923_4
        | Region::AU915
        | Region::EU868
        | Region::IN865
        | Region::KR920
        | Region::RU864
        | Region::US915 => -215.4,
        _ => return Err(anyhow!("Region not supported: {}", region)),
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
        | Region::US915 => vec![
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
        _ => return Err(anyhow!("Region not supported: {}", region)),
    };

    let usb = conf.gateway.model_flags.contains(&"USB".to_string());
    let enforce_duty_cycle = conf.gateway.model_flags.contains(&"ENFORCE_DC".to_string());

    Ok(Configuration {
        enforce_duty_cycle,
        radio_count: 2,
        clock_source: 0,
        full_duplex: false,
        lora_multi_sf_bandwidth: 125000,
        radio_config: vec![
            RadioConfig {
                tx_gain_table,
                tx_freq_min,
                tx_freq_max,
                rssi_offset,
                enable: true,
                radio_type: hal::RadioType::SX1250,
                single_input_mode: true,
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
                rssi_offset,
                enable: true,
                radio_type: hal::RadioType::SX1250,
                single_input_mode: false,
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
        com_type: match usb {
            true => ComType::Usb,
            false => ComType::Spi,
        },
        com_path: match usb {
            true => conf
                .gateway
                .com_dev_path
                .clone()
                .unwrap_or("/dev/ttyACM0".to_string()),
            false => conf
                .gateway
                .com_dev_path
                .clone()
                .unwrap_or("/dev/spidev0.0".to_string()),
        },
        i2c_path: Some(
            conf.gateway
                .i2c_dev_path
                .clone()
                .unwrap_or("/dev/i2c-1".to_string()),
        ),
        i2c_temp_sensor_addr: Some(0x39),
        sx1302_reset_pin: conf.gateway.get_sx1302_reset_pin("/dev/gpiochip0", 17),
        sx1302_power_en_pin: conf.gateway.get_sx1302_power_en_pin("/dev/gpiochip0", 18),
        sx1261_reset_pin: conf.gateway.get_sx1261_reset_pin("/dev/gpiochip0", 5),
        ..Default::default()
    })
}
