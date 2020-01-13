use std::fs;

use serde::Deserialize;
use std::time::Duration;

pub mod helpers;
pub mod vendor;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Default, Deserialize)]
pub struct Concentrator {
    pub multi_sf_channels: [u32; 8],
    #[serde(default)]
    pub lora_std: LoRaStdChannel,
    #[serde(default)]
    pub fsk: FSKChannel,
}

#[derive(Default, Deserialize)]
pub struct LoRaStdChannel {
    pub frequency: u32,
    pub bandwidth: u32,
    pub spreading_factor: u8,
}

#[derive(Default, Deserialize)]
pub struct FSKChannel {
    pub frequency: u32,
    pub datarate: u32,
    pub bandwidth: u32,
}

#[derive(Default, Deserialize)]
pub struct Gateway {
    #[serde(default)]
    pub antenna_gain: i8,
    #[serde(default)]
    pub lorawan_public: bool,
    pub model: String,
    pub gateway_id: String,
    pub concentrator: Concentrator,

    #[serde(skip)]
    pub gateway_id_bytes: Vec<u8>,
    #[serde(skip)]
    pub model_config: vendor::Configuration,
}

#[derive(Default, Deserialize)]
pub struct Concentratord {
    pub log_level: String,
    #[serde(with = "humantime_serde")]
    pub stats_interval: Duration,
    pub api: API,
}

#[derive(Default, Deserialize)]
pub struct API {
    pub event_bind: String,
    pub command_bind: String,
}

#[derive(Default, Deserialize)]
pub struct Configuration {
    pub concentratord: Concentratord,
    pub gateway: Gateway,
}

pub fn get(filename: &str) -> Configuration {
    let content = fs::read_to_string(filename).expect("Error reading config file");
    let mut config: Configuration = toml::from_str(&content).expect("Error parsing config file");

    // decode gateway id
    let bytes = hex::decode(&config.gateway.gateway_id).expect("Could not decode gateway_id");
    if bytes.len() != 8 {
        panic!("gateway_id must be exactly 8 bytes");
    }
    config.gateway.gateway_id_bytes = bytes;

    // get model configuration
    config.gateway.model_config = match config.gateway.model.as_ref() {
        "generic_as923" => vendor::generic::as923::new(false),
        "generic_as923_gps" => vendor::generic::as923::new(true),
        "generic_eu868" => vendor::generic::eu868::new(false),
        "generic_eu868_gps" => vendor::generic::eu868::new(true),
        "generic_us915" => vendor::generic::us915::new(false),
        "generic_us915_gps" => vendor::generic::us915::new(true),
        _ => panic!("unexpected gateway model: {}", config.gateway.model),
    };

    debug!("Antenna gain {} dBi", config.gateway.antenna_gain);

    return config;
}
