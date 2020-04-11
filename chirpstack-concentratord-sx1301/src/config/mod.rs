use std::fs;

use serde::Deserialize;
use std::time::Duration;

pub mod helpers;
pub mod vendor;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
}

#[derive(Default, Deserialize, Debug, PartialEq)]
pub struct FSKChannel {
    pub frequency: u32,
    pub datarate: u32,
    pub bandwidth: u32,
}

#[derive(Default, Deserialize, Clone)]
pub struct Beacon {
    pub compulsory_rfu_size: usize,
    pub frequencies: Vec<u32>,
    pub spreading_factor: u32,
    pub bandwidth: u32,
    pub tx_power: u32,
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
    pub gateway_id: String,
    pub concentrator: Concentrator,
    #[serde(default)]
    pub beacon: Beacon,

    #[serde(skip)]
    pub gateway_id_bytes: Vec<u8>,
    #[serde(skip)]
    pub model_config: vendor::Configuration,
    #[serde(skip)]
    pub config_version: String,
}

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

    // decode gateway id
    let bytes = hex::decode(&config.gateway.gateway_id).expect("Could not decode gateway_id");
    if bytes.len() != 8 {
        panic!("gateway_id must be exactly 8 bytes");
    }
    config.gateway.gateway_id_bytes = bytes;

    // get model configuration
    config.gateway.model_config = match config.gateway.model.as_ref() {
        "generic_as923" => vendor::generic::as923::new(&config),
        "generic_au915" => vendor::generic::au915::new(&config),
        "generic_cn470" => vendor::generic::cn470::new(&config),
        "generic_eu868" => vendor::generic::eu868::new(&config),
        "generic_in865" => vendor::generic::in865::new(&config),
        "generic_kr920" => vendor::generic::kr920::new(&config),
        "generic_ru864" => vendor::generic::ru864::new(&config),
        "generic_us915" => vendor::generic::us915::new(&config),
        "imst_ic880a_eu868" => vendor::imst::ic880a_eu868::new(),
        "kerlink_ifemtocell_eu868" => vendor::kerlink::ifemtocell_eu868::new(),
        "multitech_mtac_lora_h_868_eu868" => vendor::multitech::mtac_lora_h_868_eu868::new(&config),
        "multitech_mtac_lora_h_915_us915" => vendor::multitech::mtac_lora_h_915_us915::new(&config),
        "multitech_mtcap_lora_868_eu868" => vendor::multitech::mtcap_lora_868_eu868::new(),
        "multitech_mtcap_lora_915_us915" => vendor::multitech::mtcap_lora_915_us915::new(),
        "rak_2245_as923" => vendor::rak::rak2245_as923::new(&config),
        "rak_2245_au915" => vendor::rak::rak2245_au915::new(&config),
        "rak_2245_cn470" => vendor::rak::rak2245_cn470::new(&config),
        "rak_2245_eu433" => vendor::rak::rak2245_eu433::new(&config),
        "rak_2245_eu868" => vendor::rak::rak2245_eu868::new(&config),
        "rak_2245_in865" => vendor::rak::rak2245_in865::new(&config),
        "rak_2245_kr920" => vendor::rak::rak2245_kr920::new(&config),
        "rak_2245_ru864" => vendor::rak::rak2245_ru864::new(&config),
        "rak_2245_us915" => vendor::rak::rak2245_us915::new(&config),
        "rak_2246_as923" => vendor::rak::rak2246_as923::new(&config),
        "rak_2246_au915" => vendor::rak::rak2246_au915::new(&config),
        "rak_2246_eu868" => vendor::rak::rak2246_eu868::new(&config),
        "rak_2246_in865" => vendor::rak::rak2246_in865::new(&config),
        "rak_2246_kr920" => vendor::rak::rak2246_kr920::new(&config),
        "rak_2246_ru864" => vendor::rak::rak2246_ru864::new(&config),
        "rak_2246_us915" => vendor::rak::rak2246_us915::new(&config),
        "sandbox_lorago_port_eu868" => vendor::sandbox::lorago_port_eu868::new(&config),
        "sandbox_lorago_port_us915" => vendor::sandbox::lorago_port_us915::new(&config),
        "wifx_lorix_one_eu868" => vendor::wifx::lorix_one_eu868::new(&config),
        _ => panic!("unexpected gateway model: {}", config.gateway.model),
    };

    debug!("Antenna gain {} dBi", config.gateway.antenna_gain);

    return config;
}
