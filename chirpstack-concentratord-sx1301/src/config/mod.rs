use std::{env, fmt, fs};

use serde::{Deserialize, Serialize};
use std::time::Duration;

use libconcentratord::regulation::{standard, tracker::Tracker};

pub mod helpers;
pub mod vendor;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Region {
    EU868,
    US915,
    CN779,
    EU433,
    AU915,
    CN470,
    AS923,
    AS923_2,
    AS923_3,
    AS923_4,
    KR920,
    IN865,
    RU864,
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Gateway {
    pub antenna_gain: i8,
    pub lorawan_public: bool,
    pub region: Option<Region>,
    pub model: String,
    pub model_flags: Vec<String>,
    pub gateway_id: String,
    pub time_fallback_enabled: bool,
    pub concentrator: Concentrator,
    pub beacon: Beacon,
    pub location: Location,

    pub gnss_dev_path: Option<String>,
    pub com_dev_path: Option<String>,
    pub sx1301_reset_chip: Option<String>,
    pub sx1301_reset_pin: Option<u32>,

    #[serde(skip)]
    pub gateway_id_bytes: Vec<u8>,
    #[serde(skip)]
    pub model_config: vendor::Configuration,
    #[serde(skip)]
    pub config_version: String,
}

impl Default for Gateway {
    fn default() -> Self {
        Gateway {
            antenna_gain: 2,
            lorawan_public: true,
            region: None,
            model: "".into(),
            model_flags: vec![],
            gateway_id: "".into(),
            time_fallback_enabled: false,
            concentrator: Concentrator::default(),
            beacon: Beacon::default(),
            location: Location::default(),
            gnss_dev_path: None,
            com_dev_path: None,
            sx1301_reset_chip: None,
            sx1301_reset_pin: None,
            gateway_id_bytes: vec![],
            model_config: vendor::Configuration::default(),
            config_version: "".into(),
        }
    }
}

impl Gateway {
    pub fn get_sx1301_reset_pin(
        &self,
        default_chip: &str,
        default_pin: u32,
    ) -> Option<(String, u32)> {
        let chip = self
            .sx1301_reset_chip
            .clone()
            .unwrap_or(default_chip.to_string());
        let pin = self.sx1301_reset_pin.unwrap_or(default_pin);
        Some((chip, pin))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Concentratord {
    pub log_level: String,
    pub log_to_syslog: bool,
    #[serde(with = "humantime_serde")]
    pub stats_interval: Duration,
    pub disable_crc_filter: bool,
    pub api: Api,
}

impl Default for Concentratord {
    fn default() -> Self {
        Concentratord {
            log_level: "INFO".into(),
            log_to_syslog: false,
            stats_interval: Duration::from_secs(30),
            disable_crc_filter: false,
            api: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Api {
    pub event_bind: String,
    pub command_bind: String,
}

impl Default for Api {
    fn default() -> Self {
        Api {
            event_bind: "ipc:///tmp/concentratord_event".to_string(),
            command_bind: "ipc:///tmp/concentratord_command".to_string(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default = "example_configuration")]
pub struct Configuration {
    pub concentratord: Concentratord,
    pub gateway: Gateway,
}

impl Configuration {
    pub fn get_duty_cycle_tracker(&self) -> Option<Tracker> {
        match self.gateway.region {
            Some(Region::EU868) => Some(Tracker::new(
                standard::get(standard::Standard::ETSI_EN_300_220),
                self.gateway.model_config.enforce_duty_cycle,
            )),
            _ => None,
        }
    }
}

fn example_configuration() -> Configuration {
    Configuration {
        gateway: Gateway {
            lorawan_public: true,
            region: Some(Region::EU868),
            model: "rak_2245".to_string(),
            gateway_id: "0000000000000000".to_string(),
            time_fallback_enabled: true,
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
        ..Default::default()
    }
}

pub fn get(filenames: Vec<String>) -> Configuration {
    let mut content: String = String::new();

    for file_name in &filenames {
        content.push_str(&fs::read_to_string(file_name).expect("Error reading config file"));
    }

    // Replace environment variables in config.
    for (k, v) in env::vars() {
        content = content.replace(&format!("${}", k), &v);
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
        "imst_ic880a" => vendor::imst::ic880a::new(&config).unwrap(),
        "kerlink_ifemtocell" => vendor::kerlink::ifemtocell::new(&config).unwrap(),
        "multitech_mtac_lora_h_868" => vendor::multitech::mtac_lora_h_868::new(&config).unwrap(),
        "multitech_mtac_lora_h_915" => vendor::multitech::mtac_lora_h_915::new(&config).unwrap(),
        "multitech_mtcap_lora_868" => vendor::multitech::mtcap_lora_868::new(&config).unwrap(),
        "multitech_mtcap_lora_915" => vendor::multitech::mtcap_lora_915::new(&config).unwrap(),
        "pi_supply_lora_gateway_hat" => vendor::pi_supply::lora_gateway_hat::new(&config).unwrap(),
        "rak_2245" => vendor::rak::rak2245::new(&config).unwrap(),
        "rak_2246" => vendor::rak::rak2246::new(&config).unwrap(),
        "rak_2247" => vendor::rak::rak2247::new(&config).unwrap(),
        "risinghf_rhf0m301" => vendor::risinghf::rhf0m301::new(&config).unwrap(),
        "sandbox_lorago_port" => vendor::sandbox::lorago_port::new(&config).unwrap(),
        "wifx_lorix_one" => vendor::wifx::lorix_one::new(&config).unwrap(),
        _ => panic!("unexpected gateway model: {}", config.gateway.model),
    };

    debug!("Antenna gain {} dBi", config.gateway.antenna_gain);

    config
}
