use std::fs;

use serde::Deserialize;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize, Default)]
pub struct UDPBridge {
    pub log_level: String,
    #[serde(default)]
    pub log_to_syslog: bool,
    pub metrics_bind: String,
    pub servers: Vec<Server>,
}

#[derive(Deserialize, Default)]
pub struct Server {
    pub server: String,
    pub keepalive_interval_secs: u64,
    pub keepalive_max_failures: u32,
}

#[derive(Deserialize, Default)]
pub struct Concentratord {
    pub event_url: String,
    pub command_url: String,
}

#[derive(Deserialize)]
pub struct Configuration {
    pub udp_bridge: UDPBridge,
    pub concentratord: Concentratord,
}

impl Configuration {
    pub fn get(filename: &str) -> Result<Configuration, String> {
        let toml_content = match fs::read_to_string(filename) {
            Ok(v) => v,
            Err(err) => return Err(format!("read config file error: {}", err).to_string()),
        };

        let config: Configuration = match toml::from_str(&toml_content) {
            Ok(v) => v,
            Err(err) => return Err(format!("parse config file error: {}", err)),
        };

        return Ok(config);
    }
}
