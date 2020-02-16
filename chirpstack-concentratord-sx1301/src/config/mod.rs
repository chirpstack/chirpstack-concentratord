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
    pub gateway_id: String,
    pub concentrator: Concentrator,
    #[serde(default)]
    pub beacon: Beacon,

    #[serde(skip)]
    pub gateway_id_bytes: Vec<u8>,
    #[serde(skip)]
    pub model_config: vendor::Configuration,
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
        "generic_as923" => vendor::generic::as923::new(false),
        "generic_as923_gps" => vendor::generic::as923::new(true),
        "generic_au915" => vendor::generic::au915::new(false),
        "generic_au915_gps" => vendor::generic::au915::new(true),
        "generic_cn470" => vendor::generic::cn470::new(false),
        "generic_cn470_gps" => vendor::generic::cn470::new(true),
        "generic_eu868" => vendor::generic::eu868::new(false),
        "generic_eu868_gps" => vendor::generic::eu868::new(true),
        "generic_in865" => vendor::generic::in865::new(false),
        "generic_in865_gps" => vendor::generic::in865::new(true),
        "generic_kr920" => vendor::generic::kr920::new(false),
        "generic_kr920_gps" => vendor::generic::kr920::new(true),
        "generic_ru864" => vendor::generic::ru864::new(false),
        "generic_ru864_gps" => vendor::generic::ru864::new(true),
        "generic_us915" => vendor::generic::us915::new(false),
        "generic_us915_gps" => vendor::generic::us915::new(true),
        "imst_ic880a_eu868" => vendor::imst::ic880a_eu868::new(),
        "kerlink_ifemtocell_eu868" => vendor::kerlink::ifemtocell_eu868::new(),
        "multitech_mtac_lora_h_868_eu868_ap1" => vendor::multitech::mtac_lora_h_868_eu868::new(
            false,
            vendor::multitech::mtac_lora_h_868_eu868::Port::AP1,
        ),
        "multitech_mtac_lora_h_868_eu868_ap1_gps" => vendor::multitech::mtac_lora_h_868_eu868::new(
            true,
            vendor::multitech::mtac_lora_h_868_eu868::Port::AP1,
        ),
        "multitech_mtac_lora_h_868_eu868_ap2" => vendor::multitech::mtac_lora_h_868_eu868::new(
            false,
            vendor::multitech::mtac_lora_h_868_eu868::Port::AP2,
        ),
        "multitech_mtac_lora_h_868_eu868_ap2_gps" => vendor::multitech::mtac_lora_h_868_eu868::new(
            true,
            vendor::multitech::mtac_lora_h_868_eu868::Port::AP2,
        ),
        "multitech_mtac_lora_h_915_us915_ap1" => vendor::multitech::mtac_lora_h_915_us915::new(
            false,
            vendor::multitech::mtac_lora_h_915_us915::Port::AP1,
        ),
        "multitech_mtac_lora_h_915_us915_ap1_gps" => vendor::multitech::mtac_lora_h_915_us915::new(
            true,
            vendor::multitech::mtac_lora_h_915_us915::Port::AP1,
        ),
        "multitech_mtac_lora_h_915_us915_ap2" => vendor::multitech::mtac_lora_h_915_us915::new(
            false,
            vendor::multitech::mtac_lora_h_915_us915::Port::AP2,
        ),
        "multitech_mtac_lora_h_915_us915_ap2_gps" => vendor::multitech::mtac_lora_h_915_us915::new(
            true,
            vendor::multitech::mtac_lora_h_915_us915::Port::AP2,
        ),
        _ => panic!("unexpected gateway model: {}", config.gateway.model),
    };

    debug!("Antenna gain {} dBi", config.gateway.antenna_gain);

    return config;
}
