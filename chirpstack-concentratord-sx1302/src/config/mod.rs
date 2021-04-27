use std::fs;

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod helpers;
pub mod vendor;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Default, Serialize, Deserialize)]
pub struct Concentratord {
    pub log_level: String,
    #[serde(default)]
    pub log_to_syslog: bool,
    #[serde(with = "humantime_serde")]
    pub stats_interval: Duration,
    pub api: API,
}

#[derive(Default, Serialize, Deserialize)]
pub struct API {
    pub event_bind: String,
    pub command_bind: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: i16,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Gateway {
    #[serde(default)]
    pub antenna_gain: i8,
    #[serde(default)]
    pub lorawan_public: bool,

    pub model: String,
    #[serde(default)]
    pub model_flags: Vec<String>,
    #[serde(default)]
    pub reset_pin: u32,
    #[serde(default)]
    pub power_en_pin: u32,
    pub concentrator: Concentrator,
    #[serde(default)]
    pub location: Location,

    #[serde(default)]
    pub fine_timestamp: FineTimestamp,

    #[serde(skip)]
    pub model_config: vendor::Configuration,

    #[serde(skip)]
    pub config_version: String,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct Concentrator {
    pub multi_sf_channels: [u32; 8],
    #[serde(default)]
    pub lora_std: LoRaStdChannel,
    #[serde(default)]
    pub fsk: FSKChannel,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct LoRaStdChannel {
    pub frequency: u32,
    pub bandwidth: u32,
    pub spreading_factor: u8,
    #[serde(default)]
    pub implicit_header: bool,
    #[serde(default)]
    pub implicit_payload_length: u8,
    #[serde(default)]
    pub implicit_crc_enable: bool,
    #[serde(default)]
    pub implicit_coderate: String,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct FSKChannel {
    pub frequency: u32,
    pub bandwidth: u32,
    pub datarate: u32,
}

#[derive(Serialize, Deserialize)]
pub struct FineTimestamp {
    pub enable: bool,
    pub mode: String, // HIGH_CAPACITY or ALL_SF
}

impl Default for FineTimestamp {
    fn default() -> Self {
        FineTimestamp {
            enable: false,
            mode: "ALL_SF".to_string(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "example_configuration")]
pub struct Configuration {
    pub concentratord: Concentratord,
    pub gateway: Gateway,
}

fn example_configuration() -> Configuration {
    Configuration {
        concentratord: Concentratord {
            log_level: "INFO".to_string(),
            stats_interval: Duration::from_secs(30),
            api: API {
                event_bind: "ipc:///tmp/concentratord_event".to_string(),
                command_bind: "ipc:///tmp/concentratord_command".to_string(),
            },
            ..Default::default()
        },
        gateway: Gateway {
            lorawan_public: true,
            model: "rak_2287_eu868".to_string(),
            concentrator: Concentrator {
                multi_sf_channels: [
                    868100000, 868300000, 868500000, 867100000, 867300000, 867500000, 867700000,
                    867900000,
                ],
                lora_std: LoRaStdChannel {
                    frequency: 868300000,
                    bandwidth: 250000,
                    spreading_factor: 7,
                    ..Default::default()
                },
                fsk: FSKChannel {
                    frequency: 868800000,
                    bandwidth: 125000,
                    datarate: 50000,
                },
            },
            fine_timestamp: FineTimestamp {
                enable: false,
                mode: "ALL_SF".to_string(),
            },
            ..Default::default()
        },
    }
}

pub fn get(filenames: Vec<String>) -> Configuration {
    let mut content: String = String::new();

    for file_name in &filenames {
        content.push_str(&fs::read_to_string(&file_name).expect("Error reading config file"));
    }

    let mut config: Configuration = toml::from_str(&content).expect("Error parsing config file");

    // get model configuration
    config.gateway.model_config = match config.gateway.model.as_ref() {
        "semtech_sx1302c868gw1_eu868" => vendor::semtech::sx1302c868gw1_eu868::new(&config),
        "semtech_sx1302c915gw1_us915" => vendor::semtech::sx1302c915gw1_us915::new(&config),
        "rak_2287_as923" => vendor::rak::rak2287_as923::new(&config),
        "rak_2287_au915" => vendor::rak::rak2287_au915::new(&config),
        "rak_2287_eu868" => vendor::rak::rak2287_eu868::new(&config),
        "rak_2287_in865" => vendor::rak::rak2287_in865::new(&config),
        "rak_2287_kr920" => vendor::rak::rak2287_kr920::new(&config),
        "rak_2287_ru864" => vendor::rak::rak2287_ru864::new(&config),
        "rak_2287_us915" => vendor::rak::rak2287_us915::new(&config),
        _ => panic!("unexpected gateway model: {}", config.gateway.model),
    };

    debug!("Antenna gain {} dB", config.gateway.antenna_gain);

    return config;
}
