use std::fs;

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod vendor;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "example_configuration")]
pub struct Configuration {
    pub concentratord: Concentratord,
    pub gateway: Gateway,
}

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
pub struct Gateway {
    #[serde(default)]
    pub antenna_gain: i8,
    #[serde(default)]
    pub lorawan_public: bool,
    pub model: String,
    #[serde(default)]
    pub model_flags: Vec<String>,
    pub concentrator: Concentrator,
    #[serde(default)]
    pub location: Location,
    #[serde(skip)]
    pub model_config: vendor::Configuration,
    #[serde(skip)]
    pub config_version: String,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct Concentrator {
    pub channels: [Channel; 3],
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct Channel {
    pub frequency: u32,
    pub bandwidth: u32,
    pub spreading_factor: u32,
    #[serde(default)]
    pub rssi_offset: f32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: i16,
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
            model: "semtech_sx1280z3dsfgw1".to_string(),
            concentrator: Concentrator {
                channels: [
                    Channel {
                        frequency: 2403000000,
                        bandwidth: 812000,
                        spreading_factor: 12,
                        rssi_offset: 0.0,
                    },
                    Channel {
                        frequency: 2479000000,
                        bandwidth: 812000,
                        spreading_factor: 12,
                        rssi_offset: 0.0,
                    },
                    Channel {
                        frequency: 2425000000,
                        bandwidth: 812000,
                        spreading_factor: 12,
                        rssi_offset: 0.0,
                    },
                ],
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
        "semtech_sx1280z3dsfgw1" => vendor::semtech::sx1280z3dsfgw1::new(&config),
        _ => panic!("unexpected gateway model: {}", config.gateway.model),
    };

    debug!("Antenna gain {} dBi", config.gateway.antenna_gain);

    return config;
}
