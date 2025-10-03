use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::time::Duration;

use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct DevicesResponse {
    pub devices: Vec<DevicesResponseDevice>,
}

#[derive(Debug, Clone, Deserialize)]
struct DevicesResponseDevice {
    pub path: String,
    pub driver: Option<String>,
}

pub fn get_reader(server: &str) -> Result<BufReader<TcpStream>> {
    info!("Connecting to gpsd, server: {}", server);
    let stream = TcpStream::connect(server)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream);

    // VERSION
    let mut b = Vec::new();
    reader.read_until(b'\n', &mut b)?;
    debug!("Version response: {}", String::from_utf8(b.clone())?);

    // WATCH
    writer.write_all("?WATCH={\"enable\":true,\"nmea\":true,\"raw\":2};\r\n".as_bytes())?;
    writer.flush()?;

    // DEVICES
    let mut b = Vec::new();
    reader.read_until(b'\n', &mut b)?;
    debug!("Devices response: {}", String::from_utf8(b.clone())?);
    let resp: DevicesResponse = serde_json::from_slice(&b)?;

    // WATCH
    let mut b = Vec::new();
    reader.read_until(b'\n', &mut b)?;
    debug!("Watch response: {}", String::from_utf8(b.clone())?);

    for device in &resp.devices {
        if let Some(driver) = &device.driver
            && driver == "u-blox" {
                let config_str = format!("&{}=b5620601080001200001010000003294\r\n", device.path);
                debug!("Configuring uBlox device {} for NAV-TIMEGPS", device.path);
                writer.write_all(config_str.as_bytes())?;
                writer.flush()?;
                return Ok(reader);
            }
    }

    Err(anyhow!("No u-blox GNSS device found"))
}
