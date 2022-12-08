use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;

use anyhow::Result;
use log::{debug, info};

pub fn get_reader(server: &str) -> Result<BufReader<TcpStream>> {
    info!("Connecting to gpsd, server: {}", server);
    let stream = TcpStream::connect(server)?;
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

    // WATCH
    let mut b = Vec::new();
    reader.read_until(b'\n', &mut b)?;
    debug!("Watch response: {}", String::from_utf8(b.clone())?);

    Ok(reader)
}
