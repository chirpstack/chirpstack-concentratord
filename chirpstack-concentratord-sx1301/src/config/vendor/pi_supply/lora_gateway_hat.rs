use anyhow::Result;

use super::super::super::super::config;
use super::super::Configuration;

pub fn new(conf: &config::Configuration) -> Result<Configuration> {
    let mut c = super::super::rak::rak2247::new(conf)?;
    c.reset_pin = conf.gateway.get_sx1301_reset_pin("/dev/gpiochip0", 22);
    Ok(c)
}
