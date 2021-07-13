use std::convert::TryInto;
use std::ffi::CString;
use std::fs::File;
use std::ops::Add;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::ptr;
use std::time::{Duration, SystemTime};

use super::{mutex, timespec, wrapper};

/// GPS family types.
#[derive(Debug, PartialEq)]
pub enum GPSFamily {
    UBX7,
}

/// GPS coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: i16,
}

/// Type of GPS (and other GNSS) sentences.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum MessageType {
    /// neutral value
    Unknown,
    /// frame was not parsed by the system
    Ignored,
    /// system try to parse frame but failed
    Invalid,
    /// frame parsed was missing bytes
    Incomplete,

    /* NMEA messages of interest */
    /// Recommended Minimum data (time + date)
    NMEA_RMC,
    /// Global positioning system fix data (pos + alt)
    NMEA_GGA,
    /// GNSS fix data (pos + alt, sat number)
    NMEA_GNS,
    /// Time and Date
    NMEA_ZDA,

    /* NMEA message useful for time reference quality assessment */
    /// GNSS Satellite Fault Detection
    NMEA_GBS,
    /// GNSS Pseudo Range Error Statistics
    NMEA_GST,
    /// GNSS DOP and Active Satellites (sat number)
    NMEA_GSA,
    /// GNSS Satellites in View (sat SNR)
    NMEA_GSV,
    /* Misc. NMEA messages */
    /// Latitude and longitude, with time fix and status
    NMEA_GLL,
    /// Text Transmission
    NMEA_TXT,
    /// Course over ground and Ground speed
    NMEA_VTG,

    /* uBlox proprietary NMEA messages of interest */
    /// GPS Time Solution
    UBX_NAV_TIMEGPS,
    /// UTC Time Solution
    UBX_NAV_TIMEUTC,
}

impl MessageType {
    fn from_hal(msg: wrapper::gps_msg) -> Result<Self, String> {
        Ok(match msg {
            wrapper::gps_msg_UNKNOWN => MessageType::Unknown,
            wrapper::gps_msg_IGNORED => MessageType::Ignored,
            wrapper::gps_msg_INVALID => MessageType::Invalid,
            wrapper::gps_msg_INCOMPLETE => MessageType::Incomplete,
            wrapper::gps_msg_NMEA_RMC => MessageType::NMEA_RMC,
            wrapper::gps_msg_NMEA_GGA => MessageType::NMEA_GGA,
            wrapper::gps_msg_NMEA_GNS => MessageType::NMEA_GNS,
            wrapper::gps_msg_NMEA_ZDA => MessageType::NMEA_ZDA,
            wrapper::gps_msg_NMEA_GBS => MessageType::NMEA_GBS,
            wrapper::gps_msg_NMEA_GST => MessageType::NMEA_GST,
            wrapper::gps_msg_NMEA_GSA => MessageType::NMEA_GSA,
            wrapper::gps_msg_NMEA_GSV => MessageType::NMEA_GSV,
            wrapper::gps_msg_NMEA_GLL => MessageType::NMEA_GLL,
            wrapper::gps_msg_NMEA_TXT => MessageType::NMEA_TXT,
            wrapper::gps_msg_NMEA_VTG => MessageType::NMEA_VTG,
            wrapper::gps_msg_UBX_NAV_TIMEGPS => MessageType::UBX_NAV_TIMEGPS,
            wrapper::gps_msg_UBX_NAV_TIMEUTC => MessageType::UBX_NAV_TIMEUTC,
            _ => {
                return Err(format!("unexpected gps message type: {}", msg));
            }
        })
    }
}

// Time solution required for timestamp to absolute time conversion.
#[derive(Debug)]
pub struct TimeReference {
    /// System time when solution was calculated.
    pub system_time: SystemTime,
    /// Reference concentrator internal timestamp.
    pub count_us: u32,
    /// Reference GPS time (from GPS/NMEA).
    pub gps_time: SystemTime,
    /// Reference GPS epoch time (duration since 01.Jan.1980).
    pub gps_epoch: Duration,
    /// Raw clock error (eg. <1 'slow' XTAL).
    pub xtal_err: f64,
}

impl TimeReference {
    fn to_hal(&self) -> wrapper::tref {
        wrapper::tref {
            systime: self
                .system_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as wrapper::time_t,
            count_us: self.count_us,
            utc: timespec::system_time_to_timespec(&self.gps_time),
            gps: timespec::duration_to_timespec(&self.gps_epoch),
            xtal_err: self.xtal_err,
        }
    }
}

impl Default for TimeReference {
    fn default() -> Self {
        TimeReference {
            system_time: SystemTime::UNIX_EPOCH,
            count_us: 0,
            gps_time: SystemTime::UNIX_EPOCH,
            gps_epoch: Duration::new(0, 0),
            xtal_err: 0.0,
        }
    }
}

/// Configure a GPS module.
/// target_brate: target baudrate for communication (0 keeps default target baudrate).
pub fn enable(tty_path: &str, gps_family: GPSFamily, target_brate: u32) -> Result<File, String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();

    let mut fd: i32 = 0;
    let gps_family = match gps_family {
        GPSFamily::UBX7 => CString::new("ubx7").unwrap(),
    };
    let tty_path = CString::new(tty_path).unwrap();

    let ret = unsafe {
        wrapper::lgw_gps_enable(
            tty_path.into_raw(),
            gps_family.into_raw(),
            target_brate,
            &mut fd,
        )
    };
    if ret != 0 {
        return Err("lgw_gps_enable failed".to_string());
    }

    let f = unsafe { File::from_raw_fd(fd) };
    return Ok(f);
}

/// Restore GPS serial configuration and close serial device.
pub fn disable(f: File) -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();

    let fd = f.as_raw_fd();
    let ret = unsafe { wrapper::lgw_gps_disable(fd) };
    if ret != 0 {
        return Err("lgw_gps_disable failed".to_string());
    }
    return Ok(());
}

/// Parse messages coming from the GPS system (or other GNSS).
pub fn parse_nmea(b: &[u8]) -> Result<MessageType, String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let s = match CString::new(b) {
        Ok(v) => v,
        Err(_) => return Err("parse slice to CString error".to_string()),
    };

    let ret =
        unsafe { wrapper::lgw_parse_nmea(s.as_ptr(), s.as_bytes().len().try_into().unwrap()) };
    return MessageType::from_hal(ret);
}

/// Parse Ublox proprietary messages coming from the GPS system.
/// It returns the type parsed and the number of bytes parsed as UBX message if found.
pub fn parse_ubx(b: &[u8]) -> Result<(MessageType, usize), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();

    // from_vec_unchecked is used here as we are passing a slice of bytes that
    // (potentially) contain 0x00 bytes, which will panic when using ::new.
    let s = unsafe { CString::from_vec_unchecked(b.to_vec()) };

    let mut parsed_size = 0;
    let ret = unsafe {
        wrapper::lgw_parse_ubx(
            s.as_ptr(),
            s.as_bytes().len().try_into().unwrap(),
            &mut parsed_size,
        )
    };

    let msg_type = MessageType::from_hal(ret)?;
    return Ok((msg_type, parsed_size as usize));
}

/// Get the GPS solution (space & time) for the concentrator.
/// It returns the time with ns precision, duration since GPS epoch, coordinates and coordinates
/// standard deviation.
pub fn get(
    get_time: bool,
    get_location: bool,
) -> Result<(SystemTime, Duration, Coordinates, Coordinates), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();

    let mut utc: wrapper::timespec = Default::default();
    let mut gps: wrapper::timespec = Default::default();
    let mut loc: wrapper::coord_s = Default::default();
    let mut err: wrapper::coord_s = Default::default();

    let timespec_null: *mut wrapper::timespec = ptr::null_mut();
    let coords_null: *mut wrapper::coord_s = ptr::null_mut();

    let ret = unsafe {
        if get_time && get_location {
            wrapper::lgw_gps_get(&mut utc, &mut gps, &mut loc, &mut err)
        } else if get_time {
            wrapper::lgw_gps_get(&mut utc, &mut gps, coords_null, coords_null)
        } else {
            wrapper::lgw_gps_get(timespec_null, timespec_null, &mut loc, &mut err)
        }
    };
    if ret != 0 {
        return Err("lgw_gps_get failed".to_string());
    }

    let gps_time = timespec::timespec_to_system_time(&utc);
    let gps_epoch =
        Duration::from_secs(gps.tv_sec as u64) + Duration::from_nanos(gps.tv_nsec as u64);
    let loc = Coordinates {
        latitude: loc.lat,
        longitude: loc.lon,
        altitude: loc.alt,
    };
    let err = Coordinates {
        latitude: err.lat,
        longitude: err.lon,
        altitude: err.alt,
    };

    return Ok((gps_time, gps_epoch, loc, err));
}

/// Get time and position information from the serial GPS last message received.
/// Set system_time to SystemTime::UNIX_EPOCH in time_reference to trigger initial synchronization.
pub fn sync(
    t_ref: &TimeReference,
    count_us: &u32,
    gps_time: &SystemTime,
    gps_epoch: &Duration,
) -> Result<TimeReference, String> {
    let mut tref = t_ref.to_hal();

    let utc = timespec::system_time_to_timespec(gps_time);
    let gps_time = timespec::duration_to_timespec(gps_epoch);

    let ret = unsafe { wrapper::lgw_gps_sync(&mut tref, *count_us, utc, gps_time) };
    if ret != 0 {
        return Err("lgw_gps_sync failed".to_string());
    }

    let tref = TimeReference {
        system_time: SystemTime::UNIX_EPOCH.add(Duration::from_secs(tref.systime as u64)),
        count_us: tref.count_us,
        gps_time: timespec::timespec_to_system_time(&tref.utc),
        gps_epoch: timespec::timespec_to_duration(&tref.gps),
        xtal_err: tref.xtal_err,
    };

    return Ok(tref);
}

/// Convert concentrator timestamp counter value to GPS time.
pub fn cnt2time(t_ref: &TimeReference, count_us: u32) -> Result<SystemTime, String> {
    let tref = t_ref.to_hal();
    let mut utc = wrapper::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = unsafe { wrapper::lgw_cnt2utc(tref, count_us, &mut utc) };
    if ret != 0 {
        return Err("lgw_cnt2utc failed".to_string());
    }

    return Ok(timespec::timespec_to_system_time(&utc));
}

/// Convert GPS time to concentrator timestamp counter value.
pub fn time2cnt(t_ref: &TimeReference, gps_time: &SystemTime) -> Result<u32, String> {
    let tref = t_ref.to_hal();
    let utc = timespec::system_time_to_timespec(gps_time);

    let mut count_us = 0;

    let ret = unsafe { wrapper::lgw_utc2cnt(tref, utc, &mut count_us) };
    if ret != 0 {
        return Err("lgw_utc2cnt failed".to_string());
    }

    return Ok(count_us);
}

/// Convert concentrator timestamp counter value to GPS epoch.
pub fn cnt2epoch(t_ref: &TimeReference, count_us: u32) -> Result<Duration, String> {
    let tref = t_ref.to_hal();
    let mut gps_time = wrapper::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = unsafe { wrapper::lgw_cnt2gps(tref, count_us, &mut gps_time) };
    if ret != 0 {
        return Err("lgw_cnt2gps failed".to_string());
    }

    return Ok(timespec::timespec_to_duration(&gps_time));
}

/// Convert GPS epoch to concentrator timestamp counter value.
pub fn epoch2cnt(t_ref: &TimeReference, gps_epoch: &Duration) -> Result<u32, String> {
    let tref = t_ref.to_hal();
    let gps_time = timespec::duration_to_timespec(gps_epoch);
    let mut count_us = 0;

    let ret = unsafe { wrapper::lgw_gps2cnt(tref, gps_time, &mut count_us) };
    if ret != 0 {
        return Err("lgw_gps2cnt failed".to_string());
    }

    return Ok(count_us);
}
