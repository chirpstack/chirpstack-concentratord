use std::fs;

use serde::Deserialize;
use std::time::Duration;

pub mod helpers;
pub mod vendor;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Default, Deserialize)]
pub struct Concentratord {
    pub log_level: String,
    #[serde(default)]
    pub log_to_syslog: bool,
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
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: i16,
}

#[derive(Default, Deserialize)]
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
    pub precision_timestamp: PrecisionTimestamp,

    #[serde(skip)]
    pub model_config: vendor::Configuration,

    #[serde(skip)]
    pub config_version: String,
}

#[derive(Default, Deserialize, Debug, PartialEq)]
pub struct Concentrator {
    pub multi_sf_channels: [u32; 8],
    #[serde(default)]
    pub lora_std: LoRaStdChannel,
    #[serde(default)]
    pub fsk: FSKChannel,
}

#[derive(Default, Deserialize, Debug, PartialEq)]
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

#[derive(Default, Deserialize, Debug, PartialEq)]
pub struct FSKChannel {
    pub frequency: u32,
    pub bandwidth: u32,
    pub datarate: u32,
}

#[derive(Default, Deserialize)]
pub struct PrecisionTimestamp {
    pub enable: bool,
    pub max_ts_metrics: u8,
    pub nb_symbols: u8,
}

#[derive(Default, Deserialize)]
pub struct Configuration {
    pub concentratord: Concentratord,
    pub gateway: Gateway,
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
