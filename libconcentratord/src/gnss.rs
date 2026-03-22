use std::io::BufRead;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Timelike, Utc};
use log::{debug, trace, warn};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const GNSS_MAX_AGE: u32 = 30_000_000; // 30 seconds
const XERR_INIT_AVG: usize = 16;
const XERR_FILT_COEF: f64 = 256.0;

static TIME_SINCE_GPS_EPOCH: LazyLock<Mutex<Option<(GnssTimeSinceGpsEpoch, u32)>>> =
    LazyLock::new(|| Mutex::new(None));

static GNSS_DATE_TIME: LazyLock<Mutex<Option<(GnssDateTime, u32)>>> =
    LazyLock::new(|| Mutex::new(None));

static GNSS_LOCATION: LazyLock<Mutex<Option<(GnssLocation, u32)>>> =
    LazyLock::new(|| Mutex::new(None));

static STATIC_GNSS_LOCATION: LazyLock<Mutex<Option<GnssLocation>>> =
    LazyLock::new(|| Mutex::new(None));

static XTAL_CORRECT: LazyLock<Mutex<Option<XtalCorrect>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Clone)]
pub enum GnssResult {
    TimeSinceGpsEpoch(GnssTimeSinceGpsEpoch),
    DateTime(GnssDateTime),
    Location(GnssLocation),
}

#[derive(Debug, Clone)]
pub struct GnssTimeSinceGpsEpoch {
    pub time_since_gps_epoch: Duration,
}

#[derive(Debug, Clone)]
pub struct GnssDateTime {
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GnssLocation {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
}

#[derive(Debug, Clone)]
struct XtalCorrect {
    init_cpt: usize,
    init_acc: f64,
    xtal_correct: f64,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Device {
    #[default]
    None,
    TtyPath(String),
    Gpsd(String),
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum Family {
    #[default]
    Ublox,
    GenericNmea,
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

pub fn sync(result: &GnssResult, count_us_at_pps: u32) -> Result<()> {
    match result {
        GnssResult::DateTime(v) => {
            debug!(
                "Syncing GNSS datetime, count_us: {}, datetime: {}",
                count_us_at_pps, v.timestamp
            );
            let mut v = v.clone();

            // Round to second closest to the PPS. It is possible that the GNSS module calculates
            // the timestamp before the PPS is triggered, but is received after.
            v.timestamp = v
                .timestamp
                .with_nanosecond(if v.timestamp.nanosecond() > 500_000_000 {
                    // Increment by second.
                    1_000_000_000
                } else {
                    0
                })
                .ok_or_else(|| anyhow!("Strip nanoseconds error"))?;

            let mut dt = GNSS_DATE_TIME.lock().unwrap();
            let prev_dt = dt.clone();
            *dt = Some((v, count_us_at_pps));
            calculate_xtal_error(prev_dt, dt.clone().unwrap());
        }
        GnssResult::Location(v) => {
            debug!(
                "Syncing GNSS location, count_us: {}, lat: {}, lon: {}, alt: {}",
                count_us_at_pps, v.lat, v.lon, v.alt
            );
            let mut loc = GNSS_LOCATION.lock().unwrap();
            *loc = Some((v.clone(), count_us_at_pps));
        }
        GnssResult::TimeSinceGpsEpoch(v) => {
            debug!(
                "Syncing GNSS time since GPS epoch, count_us: {}, time_since_gps_epoch_ns: {}",
                count_us_at_pps,
                v.time_since_gps_epoch.as_nanos()
            );

            // Round to second closest to the PPS. It is possible that the GNSS module calculates
            // the timestamp before the PPS is triggered, but is received after.
            let mut v = v.clone();
            v.time_since_gps_epoch = Duration::from_secs(
                v.time_since_gps_epoch.as_secs()
                    + if v.time_since_gps_epoch.subsec_nanos() > 500_000_000 {
                        1
                    } else {
                        0
                    },
            );

            let mut epoch = TIME_SINCE_GPS_EPOCH.lock().unwrap();
            *epoch = Some((v, count_us_at_pps));
        }
    }

    Ok(())
}

fn calculate_xtal_error(prev_dt: Option<(GnssDateTime, u32)>, current_dt: (GnssDateTime, u32)) {
    if let Some(prev_dt) = prev_dt {
        let prev_count_us_at_pps = prev_dt.1;
        let current_count_us_at_pps = current_dt.1;
        let count_us_delta =
            (current_dt.0.timestamp - prev_dt.0.timestamp).as_seconds_f64() * 1_000_000f64;
        if count_us_delta == 0.0 {
            // Avoid divide by
            trace!("count_us_delta == 0, skipping xtal error correction");
            return;
        }

        let (count_us_diff, _) = current_count_us_at_pps.overflowing_sub(prev_count_us_at_pps);
        if count_us_diff == 0 {
            // PPS has not been triggered yet
            trace!("count_us_diff == 0, skipping xtal error correction");
            return;
        }

        let xtal_error = count_us_diff as f64 / count_us_delta;
        debug!(
            "xtal error calculated, error: {:.12}, prev_count_us: {}, current_count_us: {}",
            xtal_error, prev_count_us_at_pps, current_count_us_at_pps
        );
        if xtal_error > 1.00001 || xtal_error < 0.99999 {
            warn!(
                "xtal error out of expected range, xtal_error: {:.6}",
                xtal_error
            );
            return;
        }

        let mut xtal_correct = XTAL_CORRECT.lock().unwrap();
        if let Some(xtal_correct) = xtal_correct.as_mut() {
            if xtal_correct.init_cpt < XERR_INIT_AVG {
                xtal_correct.init_cpt += 1;
                xtal_correct.init_acc += xtal_error;
            } else if xtal_correct.init_cpt == XERR_INIT_AVG {
                xtal_correct.xtal_correct = XERR_INIT_AVG as f64 / xtal_correct.init_acc;
                debug!(
                    "xtal correction calculated, xtal_correct: {:.12}",
                    xtal_correct.xtal_correct
                );
            } else {
                let x = 1.0 / xtal_error;
                xtal_correct.xtal_correct = xtal_correct.xtal_correct
                    - xtal_correct.xtal_correct / XERR_FILT_COEF
                    + x / XERR_FILT_COEF;
                debug!(
                    "xtal correction calculated, xtal_correct: {:.12}",
                    xtal_correct.xtal_correct
                );
            }
        } else {
            *xtal_correct = Some(XtalCorrect {
                init_cpt: 1,
                init_acc: xtal_error,
                xtal_correct: 1.0,
            })
        }
    }
}

pub fn set_static_location(lat: f64, lon: f64, alt: f32) {
    if lat == 0.0 && lon == 0.0 && alt == 0.0 {
        return;
    }

    let mut loc = STATIC_GNSS_LOCATION.lock().unwrap();
    *loc = Some(GnssLocation { lat, lon, alt });
}

pub fn count_to_time(count_us: u32) -> Option<DateTime<Utc>> {
    let mut gnss_dt_mux = GNSS_DATE_TIME.lock().unwrap();
    if let Some((gnss_dt, gnss_dt_count_us)) = gnss_dt_mux.as_ref() {
        let (count_us_diff, _) = count_us.overflowing_sub(*gnss_dt_count_us);
        if count_us_diff > GNSS_MAX_AGE {
            debug!("GNSS timestamp is too old");
            *gnss_dt_mux = None;
            return None;
        }
        Some(gnss_dt.timestamp + Duration::from_micros(count_us_diff as u64))
    } else {
        trace!("No GNSS timestamp available");
        None
    }
}

pub fn count_to_epoch(count_us: u32) -> Option<Duration> {
    let mut gps_epoch_mux = TIME_SINCE_GPS_EPOCH.lock().unwrap();

    if let Some((gps_epoch, gps_epoch_count_us)) = gps_epoch_mux.as_ref() {
        let (count_us_diff, _) = count_us.overflowing_sub(*gps_epoch_count_us);
        if count_us_diff > GNSS_MAX_AGE {
            *gps_epoch_mux = None;
            return None;
        }

        Some(gps_epoch.time_since_gps_epoch + Duration::from_micros(count_us_diff as u64))
    } else {
        None
    }
}

pub fn epoch_to_count(gps_epoch_now: Duration) -> Option<u32> {
    let gps_epoch_mux = TIME_SINCE_GPS_EPOCH.lock().unwrap();
    if let Some((gps_epoch, gps_epoch_count_us)) = gps_epoch_mux.as_ref() {
        gps_epoch_now
            .checked_sub(gps_epoch.time_since_gps_epoch)
            .map(|diff| gps_epoch_count_us.wrapping_add(diff.as_micros() as u32))
    } else {
        None
    }
}

pub fn get_location(count_us: u32) -> Option<GnssLocation> {
    let mut gnss_location_mux = GNSS_LOCATION.lock().unwrap();
    if let Some((gnss_location, gnss_location_count_us)) = gnss_location_mux.as_ref() {
        let (count_us_diff, _) = count_us.overflowing_sub(*gnss_location_count_us);
        if count_us_diff > GNSS_MAX_AGE {
            *gnss_location_mux = None;
            debug!("GNSS location is too old");
        } else {
            return Some(gnss_location.clone());
        }
    }

    STATIC_GNSS_LOCATION.lock().unwrap().clone()
}

pub fn get_location_last_updated_at() -> Option<DateTime<Utc>> {
    let gnss_location = GNSS_LOCATION.lock().unwrap();
    if let Some((_, gnss_location_count_us)) = gnss_location.as_ref() {
        count_to_time(*gnss_location_count_us)
    } else {
        None
    }
}

pub fn get_xtal_correct() -> f64 {
    if let Some(xtal_correct) = XTAL_CORRECT.lock().unwrap().as_ref() {
        xtal_correct.xtal_correct
    } else {
        1.0
    }
}

fn parse_nmea(b: &[u8]) -> Result<Option<GnssResult>> {
    let v = nmea::parse_bytes(b).map_err(|e| anyhow!("NMEA parse error: {:?}", e))?;

    match &v {
        nmea::ParseResult::RMC(v) => handle_nmea_rcm(v),
        nmea::ParseResult::GGA(v) => handle_nmea_gga(v),
        _ => Ok(None),
    }
}

fn parse_ubx(b: &[u8]) -> Result<Option<GnssResult>> {
    let mut parser = ublox::Parser::default();
    let mut it = parser.consume_ubx(b);
    match it.next() {
        Some(Ok(ublox::UbxPacket::Proto23(ublox::proto23::PacketRef::NavTimeGps(v)))) => {
            handle_ubx_nav_timegps(v)
        }
        _ => Ok(None),
    }
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
