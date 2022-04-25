use std::fs;

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod helpers;
pub mod vendor;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct FSKChannel {
    pub frequency: u32,
    pub datarate: u32,
    pub bandwidth: u32,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Beacon {
    pub compulsory_rfu_size: usize,
    pub frequencies: Vec<u32>,
    pub spreading_factor: u32,
    pub bandwidth: u32,
    pub tx_power: u32,
}

#[derive(Default, Serialize, Deserialize, Clone)]
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
    pub gateway_id: String,
    pub concentrator: Concentrator,
    #[serde(default)]
    pub beacon: Beacon,
    #[serde(default)]
    pub location: Location,

    #[serde(skip)]
    pub gateway_id_bytes: Vec<u8>,
    #[serde(skip)]
    pub model_config: vendor::Configuration,
    #[serde(skip)]
    pub config_version: String,
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
            model: "rak_2245_eu868".to_string(),
            gateway_id: "0000000000000000".to_string(),
            concentrator: Concentrator {
                multi_sf_channels: [
                    868100000, 868300000, 868500000, 867100000, 867300000, 867500000, 867700000,
                    867900000,
                ],
                lora_std: LoRaStdChannel {
                    frequency: 868300000,
                    bandwidth: 250000,
                    spreading_factor: 7,
                },
                fsk: FSKChannel {
                    frequency: 868800000,
                    bandwidth: 125000,
                    datarate: 50000,
                },
            },
            beacon: Beacon {
                compulsory_rfu_size: 2,
                frequencies: vec![869525000],
                spreading_factor: 9,
                bandwidth: 125000,
                tx_power: 14,
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
        "imst_ic880a_eu868" => vendor::imst::ic880a_eu868::new(&config),
        "imst_ic880a_in865" => vendor::imst::ic880a_in865::new(&config),
        "imst_ic880a_ru864" => vendor::imst::ic880a_ru864::new(&config),
        "kerlink_ifemtocell_eu868" => vendor::kerlink::ifemtocell_eu868::new(),
        "multitech_mtac_lora_h_868_eu868" => vendor::multitech::mtac_lora_h_868_eu868::new(&config),
        "multitech_mtac_lora_h_915_us915" => vendor::multitech::mtac_lora_h_915_us915::new(&config),
        "multitech_mtcap_lora_868_eu868" => vendor::multitech::mtcap_lora_868_eu868::new(),
        "multitech_mtcap_lora_915_us915" => vendor::multitech::mtcap_lora_915_us915::new(),
        "pi_supply_lora_gateway_hat_au915" => {
            vendor::pi_supply::lora_gateway_hat_au915::new(&config)
        }
        "pi_supply_lora_gateway_hat_eu868" => {
            vendor::pi_supply::lora_gateway_hat_eu868::new(&config)
        }
        "pi_supply_lora_gateway_hat_us915" => {
            vendor::pi_supply::lora_gateway_hat_us915::new(&config)
        }
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
        "rak_2246_cn470" => vendor::rak::rak2246_cn470::new(&config),
        "rak_2246_eu433" => vendor::rak::rak2246_eu433::new(&config),
        "rak_2246_eu868" => vendor::rak::rak2246_eu868::new(&config),
        "rak_2246_in865" => vendor::rak::rak2246_in865::new(&config),
        "rak_2246_kr920" => vendor::rak::rak2246_kr920::new(&config),
        "rak_2246_ru864" => vendor::rak::rak2246_ru864::new(&config),
        "rak_2246_us915" => vendor::rak::rak2246_us915::new(&config),
        "rak_2247_as923" => vendor::rak::rak2247_as923::new(&config),
        "rak_2247_au915" => vendor::rak::rak2247_au915::new(&config),
        "rak_2247_cn470" => vendor::rak::rak2247_cn470::new(&config),
        "risinghf_rhf0m301_eu868" => vendor::risinghf::rhf0m301_eu868::new(&config),
        "risinghf_rhf0m301_us915" => vendor::risinghf::rhf0m301_us915::new(&config),
        "sandbox_lorago_port_eu868" => vendor::sandbox::lorago_port_eu868::new(&config),
        "sandbox_lorago_port_us915" => vendor::sandbox::lorago_port_us915::new(&config),
        "wifx_lorix_one_eu868" => vendor::wifx::lorix_one_eu868::new(&config),
        _ => panic!("unexpected gateway model: {}", config.gateway.model),
    };

    debug!("Antenna gain {} dBi", config.gateway.antenna_gain);

    return config;
}
