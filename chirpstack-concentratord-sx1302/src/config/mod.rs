use std::{fmt, fs};

use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[derive(Default, Serialize, Deserialize)]
pub struct Concentratord {
    pub log_level: String,
    #[serde(default)]
    pub log_to_syslog: bool,
    #[serde(with = "humantime_serde")]
    pub stats_interval: Duration,
    #[serde(default)]
    pub disable_crc_filter: bool,
    pub api: Api,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Api {
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

    pub region: Option<Region>,
    pub model: String,
    #[serde(default)]
    pub model_flags: Vec<String>,

    #[serde(default)]
    pub time_fallback_enabled: bool,
    pub concentrator: Concentrator,
    #[serde(default)]
    pub beacon: Beacon,
    #[serde(default)]
    pub location: Location,

    #[serde(default)]
    pub fine_timestamp: FineTimestamp,

    pub sx1302_reset_pin: Option<u32>,
    pub sx1302_power_en_pin: Option<u32>,
    pub sx1261_reset_pin: Option<u32>,
    pub gnss_dev_path: Option<String>,
    pub com_dev_path: Option<String>,
    pub i2c_dev_path: Option<String>,

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

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Beacon {
    pub compulsory_rfu_size: usize,
    pub frequencies: Vec<u32>,
    pub spreading_factor: u32,
    pub bandwidth: u32,
    pub tx_power: u32,
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
            api: Api {
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
        content.push_str(&fs::read_to_string(file_name).expect("Error reading config file"));
    }

    let mut config: Configuration = toml::from_str(&content).expect("Error parsing config file");

    // get model configuration
    config.gateway.model_config = match config.gateway.model.as_ref() {
        "dragino_pg1302" => vendor::dragino::pg1302::new(&config).unwrap(),
        "multitech_mtac_003e00" => vendor::multitech::mtac_003e00::new(&config).unwrap(),
        "multitech_mtac_003u00" => vendor::multitech::mtac_003u00::new(&config).unwrap(),
        "multitech_mtcap3_003e00" => vendor::multitech::mtcap3_003e00::new(&config).unwrap(),
        "multitech_mtcap3_003u00" => vendor::multitech::mtcap3_003u00::new(&config).unwrap(),
        "rak_2287" => vendor::rak::rak2287::new(&config).unwrap(),
        "rak_5146" => vendor::rak::rak5146::new(&config).unwrap(),
        "seeed_wm1302" => vendor::seeed::wm1302::new(&config).unwrap(),
        "semtech_sx1302c490gw1" => vendor::semtech::sx1302c490gw1::new(&config).unwrap(),
        "semtech_sx1302c868gw1" => vendor::semtech::sx1302c868gw1::new(&config).unwrap(),
        "semtech_sx1302c915gw1" => vendor::semtech::sx1302c915gw1::new(&config).unwrap(),
        "semtech_sx1302css868gw1" => vendor::semtech::sx1302css868gw1::new(&config).unwrap(),
        "semtech_sx1302css915gw1" => vendor::semtech::sx1302css915gw1::new(&config).unwrap(),
        "semtech_sx1302css923gw1" => vendor::semtech::sx1302css923gw1::new(&config).unwrap(),
        "waveshare_sx1302_lorawan_gateway_hat" => {
            vendor::waveshare::sx1302_lorawan_gateway_hat::new(&config).unwrap()
        }
        _ => panic!("unexpected gateway model: {}", config.gateway.model),
    };

    debug!("Antenna gain {} dBi", config.gateway.antenna_gain);

    config
}
