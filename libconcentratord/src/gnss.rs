use std::sync::{LazyLock, RwLock};
use std::time::Duration;
use std::{i128, io::BufRead};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, trace};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const GNSS_MAX_AGE: u32 = 30_000_000; // 30 seconds

static TIME_SINCE_GPS_EPOCH: LazyLock<RwLock<Option<(GnssTimeSinceGpsEpoch, u32)>>> =
    LazyLock::new(|| RwLock::new(None));

static GNSS_DATE_TIME: LazyLock<RwLock<Option<(GnssDateTime, u32)>>> =
    LazyLock::new(|| RwLock::new(None));

static GNSS_LOCATION: LazyLock<RwLock<Option<(GnssLocation, u32)>>> =
    LazyLock::new(|| RwLock::new(None));

static STATIC_GNSS_LOCATION: LazyLock<RwLock<Option<GnssLocation>>> =
    LazyLock::new(|| RwLock::new(None));

#[derive(Debug, Clone, PartialEq)]
pub enum GnssResult {
    TimeSinceGpsEpoch(GnssTimeSinceGpsEpoch),
    DateTime(GnssDateTime),
    Location(GnssLocation),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GnssTimeSinceGpsEpoch {
    pub time_since_gps_epoch: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GnssDateTime {
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GnssLocation {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Device {
    #[default]
    None,
    TtyPath(String),
    Gpsd(String),
}

impl Device {
    pub fn new(path: &str) -> Device {
        if path.is_empty() {
            return Device::None;
        }

        if let Some(host) = path.strip_prefix("gpsd://") {
            return Device::Gpsd(host.to_string());
        }

        Device::TtyPath(path.to_string())
    }
}

impl<'de> Deserialize<'de> for Device {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Device::new(&s))
    }
}

impl Serialize for Device {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&match self {
            Device::None => "".to_string(),
            Device::TtyPath(v) => v.to_string(),
            Device::Gpsd(v) => format!("gpsd://{}", v),
        })
    }
}

pub fn read(gps_reader: &mut Box<dyn BufRead>) -> Result<Option<GnssResult>> {
    let mut buffer = vec![0; 1];
    gps_reader
        .read_exact(&mut buffer)
        .context("Read from GPS")?;

    match buffer[0] {
        // ubx
        0xb5 => {
            // We need to read 5 additional bytes for the header + PL length.
            buffer.resize(6, 0);
            gps_reader
                .read_exact(&mut buffer[1..])
                .context("Read from GPS")?;

            // Parse PL length and read additional payload.
            let len: usize = u16::from_le_bytes([buffer[4], buffer[5]]).into();
            buffer.resize(6 + len + 2, 0);
            gps_reader
                .read_exact(&mut buffer[6..])
                .context("Read from GPS")?;

            parse_ubx(&buffer)
        }
        0x24 => {
            gps_reader
                .read_until(b'\n', &mut buffer)
                .context("Read from GPS")?;

            parse_nmea(&buffer)
        }
        _ => Ok(None),
    }
}

pub fn sync(result: &GnssResult, count_us: u32) -> Result<()> {
    match result {
        GnssResult::DateTime(v) => {
            debug!(
                "Syncing GNSS datetime, count_us: {}, datetime: {}",
                count_us, v.timestamp
            );
            let mut dt = GNSS_DATE_TIME.write().unwrap();
            *dt = Some((v.clone(), count_us));
        }
        GnssResult::Location(v) => {
            debug!(
                "Syncing GNSS location, count_us: {}, lat: {}, lon: {}, alt: {}",
                count_us, v.lat, v.lon, v.alt
            );
            let mut loc = GNSS_LOCATION.write().unwrap();
            *loc = Some((v.clone(), count_us));
        }
        GnssResult::TimeSinceGpsEpoch(v) => {
            debug!(
                "Syncing GNSS time since GPS epoch, count_us: {}, time_since_gps_epoch_sec: {}",
                count_us,
                v.time_since_gps_epoch.as_secs()
            );
            let mut epoch = TIME_SINCE_GPS_EPOCH.write().unwrap();
            *epoch = Some((v.clone(), count_us));
        }
    }

    Ok(())
}

pub fn set_static_location(lat: f64, lon: f64, alt: f32) {
    if lat == 0.0 && lon == 0.0 && alt == 0.0 {
        return;
    }

    let mut loc = STATIC_GNSS_LOCATION.write().unwrap();
    *loc = Some(GnssLocation { lat, lon, alt });
}

pub fn count_to_time(count_us: u32) -> Option<DateTime<Utc>> {
    let gnss_dt = GNSS_DATE_TIME.read().unwrap();
    if let Some((gnss_dt, gnss_dt_count_us)) = gnss_dt.as_ref() {
        let (count_us_diff, _) = count_us.overflowing_sub(*gnss_dt_count_us);
        if count_us_diff > GNSS_MAX_AGE {
            debug!("GNSS timestamp is too old");
            return None;
        }
        Some(gnss_dt.timestamp + Duration::from_micros(count_us_diff as u64))
    } else {
        trace!("No GNSS timestamp available");
        None
    }
}

pub fn count_to_epoch(count_us: u32) -> Option<Duration> {
    let gps_epoch = TIME_SINCE_GPS_EPOCH.read().unwrap();
    if let Some((gps_epoch, gps_epoch_count_us)) = gps_epoch.as_ref() {
        let (count_us_diff, _) = count_us.overflowing_sub(*gps_epoch_count_us);
        if count_us_diff > GNSS_MAX_AGE {
            return None;
        }

        Some(gps_epoch.time_since_gps_epoch + Duration::from_micros(count_us_diff as u64))
    } else {
        None
    }
}

pub fn epoch_to_count(gps_epoch_now: Duration) -> Option<u32> {
    let gps_epoch = TIME_SINCE_GPS_EPOCH.read().unwrap();
    if let Some((gps_epoch, gps_epoch_count_us)) = gps_epoch.as_ref() {
        if let Some(diff) = gps_epoch_now.checked_sub(gps_epoch.time_since_gps_epoch) {
            Some(gps_epoch_count_us.wrapping_add(diff.as_micros() as u32))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_location(count_us: u32) -> Option<GnssLocation> {
    let gnss_location = GNSS_LOCATION.read().unwrap();
    if let Some((gnss_location, gnss_location_count_us)) = gnss_location.as_ref() {
        let (count_us_diff, _) = count_us.overflowing_sub(*gnss_location_count_us);
        if count_us_diff < GNSS_MAX_AGE {
            return Some(gnss_location.clone());
        }
    }

    let gnss_location = STATIC_GNSS_LOCATION.read().unwrap();
    gnss_location.clone()
}

pub fn get_location_last_updated_at() -> Option<DateTime<Utc>> {
    let gnss_location = GNSS_LOCATION.read().unwrap();
    if let Some((_, gnss_location_count_us)) = gnss_location.as_ref() {
        count_to_time(*gnss_location_count_us)
    } else {
        None
    }
}

fn parse_nmea(b: &[u8]) -> Result<Option<GnssResult>> {
    let v = nmea::parse_bytes(b).map_err(|e| anyhow!("NMEA parse error: {:?}", e))?;

    match &v {
        nmea::ParseResult::RMC(v) => handle_nmea_rcm(&v),
        nmea::ParseResult::GGA(v) => handle_nmea_gga(&v),
        _ => Ok(None),
    }
}

fn parse_ubx(b: &[u8]) -> Result<Option<GnssResult>> {
    let mut parser = ublox::Parser::default();
    let mut it = parser.consume_ubx(&b);
    loop {
        return match it.next() {
            Some(Ok(packet)) => match packet {
                ublox::UbxPacket::Proto23(ublox::proto23::PacketRef::NavTimeGps(v)) => {
                    handle_ubx_nav_timegps(v)
                }
                _ => Ok(None),
            },
            _ => break,
        };
    }

    Ok(None)
}

fn handle_nmea_rcm(v: &nmea::sentences::RmcData) -> Result<Option<GnssResult>> {
    if v.fix_time.is_none() || v.fix_date.is_none() {
        return Ok(None);
    }

    let fix_time = v.fix_time.unwrap_or_default();
    let fix_date = v.fix_date.unwrap_or_default();
    let ts = fix_date.and_time(fix_time).and_utc();

    Ok(Some(GnssResult::DateTime(GnssDateTime { timestamp: ts })))
}

fn handle_nmea_gga(v: &nmea::sentences::GgaData) -> Result<Option<GnssResult>> {
    if v.latitude.is_none() || v.longitude.is_none() || v.altitude.is_none() {
        return Ok(None);
    }

    Ok(Some(GnssResult::Location(GnssLocation {
        lat: v.latitude.unwrap_or_default(),
        lon: v.longitude.unwrap_or_default(),
        alt: v.altitude.unwrap_or_default(),
    })))
}

fn handle_ubx_nav_timegps(v: ublox::nav_time_gps::NavTimeGpsRef) -> Result<Option<GnssResult>> {
    if !v.valid().contains(
        ublox::nav_time_gps::NavTimeGpsFlags::VALID_TOW
            | ublox::nav_time_gps::NavTimeGpsFlags::VALID_WKN,
    ) {
        return Ok(None);
    }

    let week_ns = v.week() as i128 * 7 * 24 * 60 * 60 * 1_000_000_000;
    let itow_ns = v.itow() as i128 * 1_000_000;
    let ftow_ns = v.ftow() as i128;

    let total_ns = week_ns + itow_ns + ftow_ns;

    if total_ns < 0 {
        return Err(anyhow!("Negative GPS time"));
    }

    Ok(Some(GnssResult::TimeSinceGpsEpoch(GnssTimeSinceGpsEpoch {
        time_since_gps_epoch: Duration::from_nanos(total_ns as u64),
    })))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_device_none() {
        assert_eq!(Device::None, Device::new(""));
    }

    #[test]
    fn test_device_tty_path() {
        assert_eq!(
            Device::TtyPath("/dev/ttyAMA0".to_string()),
            Device::new("/dev/ttyAMA0")
        );
    }

    #[test]
    fn test_device_gpsd() {
        assert_eq!(
            Device::Gpsd("localhost:2947".to_string()),
            Device::new("gpsd://localhost:2947")
        );
    }
}
