use std::cmp::Ordering;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::Receiver;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use chrono::DateTime;
use chrono::offset::Utc;

use libconcentratord::{gnss, gpsd, signals::Signal};
use libloragw_sx1301::{gps, hal};

static GPS_TIME_REF: LazyLock<Mutex<gps::TimeReference>> =
    LazyLock::new(|| Mutex::new(Default::default()));
static STATIC_GPS_COORDS: LazyLock<Mutex<Option<gps::Coordinates>>> =
    LazyLock::new(|| Mutex::new(None));
static GPS_COORDS: LazyLock<Mutex<Option<gps::Coordinates>>> = LazyLock::new(|| Mutex::new(None));
static GPS_COORDS_ERROR: LazyLock<Mutex<gps::Coordinates>> = LazyLock::new(|| {
    Mutex::new(gps::Coordinates {
        latitude: 0.0,
        longitude: 0.0,
        altitude: 0,
    })
});
static GPS_COORDS_LAST_UPDATE: LazyLock<Mutex<Option<DateTime<Utc>>>> =
    LazyLock::new(|| Mutex::new(None));
static GPS_TIME_REF_VALID: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static XTAL_CORRECT_OK: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static XTAL_CORRECT: LazyLock<Mutex<f64>> = LazyLock::new(|| Mutex::new(1.0));

const XERR_INIT_AVG: isize = 128;
const XERR_FILT_COEF: f64 = 256.0;

pub fn set_static_gps_coords(lat: f64, lon: f64, alt: i16) {
    let mut static_gps_coords = STATIC_GPS_COORDS.lock().unwrap();

    if lat != 0.0 || lon != 0.0 || alt != 0 {
        *static_gps_coords = Some(gps::Coordinates {
            latitude: lat,
            longitude: lon,
            altitude: alt,
        })
    } else {
        *static_gps_coords = None;
    }
}

pub fn gps_loop(gps_device: gnss::Device, stop_receive: Receiver<Signal>) -> Result<()> {
    debug!("Starting GPS loop");

    let mut gps_reader: Box<dyn BufRead> = match gps_device {
        gnss::Device::TtyPath(tty_path) => {
            info!("Enabling GPS device, tty_path: {}", tty_path);
            let gps_file = gps::enable(&tty_path, gps::GPSFamily::UBX7, 0)
                .expect("could not open gps tty path for gps sync");
            Box::new(BufReader::new(gps_file)) as Box<dyn BufRead>
        }
        gnss::Device::Gpsd(gpsd_host) => {
            info!("Starting gpsd reader, server: localhost:2947");
            Box::new(gpsd::get_reader(&gpsd_host).expect("could not open gpsd reader"))
                as Box<dyn BufRead>
        }
        gnss::Device::None => {
            warn!("No GPS device configured");
            return Ok(());
        }
    };

    loop {
        if let Ok(v) = stop_receive.recv_timeout(Duration::from_millis(0)) {
            debug!("Received stop signal, signal: {}", v);
            return Ok(());
        }

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

                // Ignore messages other than "B5620120"
                if !buffer[0..4].eq(&[0xb5, 0x62, 0x01, 0x20]) {
                    continue;
                }

                match gps::parse_ubx(&buffer) {
                    Ok((m_type, _)) => {
                        if m_type == gps::MessageType::UBX_NAV_TIMEGPS {
                            debug!("Processing u-blox NAV-TIMEGPS");
                            gps_process_sync();
                        }
                    }
                    Err(err) => {
                        error!("Parse ubx error, error: {}", err);
                        continue;
                    }
                };
            }
            // nmea
            0x24 => {
                gps_reader
                    .read_until(b'\n', &mut buffer)
                    .context("Read from GPS")?;

                match gps::parse_nmea(&buffer[..buffer.len() - 1]) {
                    Ok(m_type) => {
                        if m_type == gps::MessageType::NMEA_RMC {
                            debug!("Processing NMEA RMC");
                            gps_process_coords();
                        }
                    }
                    Err(err) => {
                        error!("Parse nmea string error, error: {}", err);
                        continue;
                    }
                }
            }
            _ => {
                // No error logging here. When an unknown ubx message header is
                // received, we first need to find the next nmea or ubx
                // identifier.
            }
        }
    }
}

pub fn gps_validate_loop(stop_receive: Receiver<Signal>) -> Result<()> {
    info!("Starting GPS validation loop");

    let mut init_cpt: isize = 0;
    let mut init_acc: f64 = 0.0;

    loop {
        // Instead of a 1s sleep, we receive from the stop channel with a
        // timeout of 1 second.
        if let Ok(v) = stop_receive.recv_timeout(Duration::from_secs(1)) {
            debug!("Received stop signal, signal: {}", v);
            return Ok(());
        }

        // Scope to make sure the mutex guard is dereferenced after validation.
        {
            let time_ref = GPS_TIME_REF.lock().unwrap();
            let mut gps_ref_valid = GPS_TIME_REF_VALID.lock().unwrap();
            let mut xtal_correct_ok = XTAL_CORRECT_OK.lock().unwrap();
            let mut xtal_correct = XTAL_CORRECT.lock().unwrap();

            // validate the age of last gps time reference
            let systime_diff = match SystemTime::now().duration_since(time_ref.system_time) {
                Ok(v) => v,
                Err(err) => {
                    error!(
                        "Get duration since last time reference update error, error: {}",
                        err
                    );
                    continue;
                }
            };
            if systime_diff > Duration::from_secs(30) {
                *gps_ref_valid = false;

                warn!("GPS time reference is not valid, age: {:?}", systime_diff);
            } else {
                *gps_ref_valid = true;
                trace!("GPS time reference is valid");
            }

            // manage xtal correction
            if !(*gps_ref_valid) {
                *xtal_correct_ok = false;
                *xtal_correct = 1.0;
                init_cpt = 0;
                init_acc = 0.0;
            } else {
                match init_cpt.cmp(&XERR_INIT_AVG) {
                    Ordering::Less => {
                        init_acc += time_ref.xtal_err;
                        init_cpt += 1;
                        trace!(
                            "Initial accumulation, xtal_err: {}, init_acc: {}, init_cpt: {}",
                            time_ref.xtal_err, init_acc, init_cpt
                        );
                    }
                    Ordering::Equal => {
                        *xtal_correct = XERR_INIT_AVG as f64 / init_acc;
                        *xtal_correct_ok = true;
                        init_cpt += 1;
                        trace!(
                            "Initial average calculation, xtal_correct: {}, init_cpt: {}",
                            xtal_correct, init_cpt
                        );
                    }
                    Ordering::Greater => {
                        let x = 1.0 / time_ref.xtal_err;
                        *xtal_correct =
                            *xtal_correct - *xtal_correct / XERR_FILT_COEF + x / XERR_FILT_COEF;
                        trace!(
                            "Tracking with low-pass filter, x: {}, xtal_correct: {}",
                            x, xtal_correct
                        );
                    }
                }
            }
        }
    }
}

pub fn cnt2time(count_us: u32) -> Result<SystemTime> {
    let gps_ref_valid = GPS_TIME_REF_VALID.lock().unwrap();
    if !(*gps_ref_valid) {
        return Err(anyhow!("gps_ref_valid = false"));
    }
    let gps_time_ref = GPS_TIME_REF.lock().unwrap();

    gps::cnt2time(&gps_time_ref, count_us)
}

pub fn cnt2epoch(count_us: u32) -> Result<Duration> {
    let gps_ref_valid = GPS_TIME_REF_VALID.lock().unwrap();
    if !(*gps_ref_valid) {
        return Err(anyhow!("gps_ref_valid = false"));
    }
    let gps_time_ref = GPS_TIME_REF.lock().unwrap();

    gps::cnt2epoch(&gps_time_ref, count_us)
}

pub fn epoch2cnt(gps_epoch: &Duration) -> Result<u32> {
    let gps_ref_valid = GPS_TIME_REF_VALID.lock().unwrap();
    if !(*gps_ref_valid) {
        return Err(anyhow!("gps_ref_valid = false"));
    }
    let gps_time_ref = GPS_TIME_REF.lock().unwrap();

    gps::epoch2cnt(&gps_time_ref, gps_epoch)
}

pub fn get_coords() -> Option<gps::Coordinates> {
    let gps_time_ref_valid = GPS_TIME_REF_VALID.lock().unwrap();
    let coords = GPS_COORDS.lock().unwrap();
    let static_gps_coords = STATIC_GPS_COORDS.lock().unwrap();

    // In case the gps time reference is invalid or no gps coordinates
    // are available, use static coords (which can be None).
    if !(*gps_time_ref_valid) || coords.is_none() {
        return *static_gps_coords;
    }

    *coords
}

pub fn get_coords_last_update() -> Option<DateTime<Utc>> {
    let coords_last_update = GPS_COORDS_LAST_UPDATE.lock().unwrap();
    *coords_last_update
}

pub fn get_gps_epoch() -> Result<Duration> {
    if !(*GPS_TIME_REF_VALID.lock().unwrap()) {
        return Err(anyhow!("gps time reference not available"));
    }

    Ok(GPS_TIME_REF.lock().unwrap().gps_epoch)
}

fn gps_process_sync() {
    let (gps_time, gps_epoch, _, _) = match gps::get(true, false) {
        Ok(v) => v,
        Err(err) => {
            debug!("Get gps time failed, error: {}", err);
            return;
        }
    };

    let trig_cnt = match hal::get_trigcnt() {
        Ok(v) => v,
        Err(err) => {
            error!("Get internal concentrator counter error, error: {}", err);
            return;
        }
    };

    let mut time_reference = GPS_TIME_REF.lock().unwrap();

    *time_reference = match gps::sync(&time_reference, &trig_cnt, &gps_time, &gps_epoch) {
        Ok(v) => v,
        Err(err) => {
            // On initial start, it is expected that this will fail a couple of times.
            debug!("GPS sync error, error: {}", err);
            return;
        }
    };

    let sys_time: DateTime<Utc> = time_reference.system_time.into();
    let gps_time: DateTime<Utc> = time_reference.gps_time.into();

    trace!(
        "GPS time sync completed, count_us: {}, system_time: {} (UTC), gps_time: {} (UTC), gps_epoch: {:?}, xtal_err: {}",
        time_reference.count_us,
        sys_time.format("%Y-%m-%d %T"),
        gps_time.format("%Y-%m-%d %T"),
        time_reference.gps_epoch,
        time_reference.xtal_err,
    );
}

fn gps_process_coords() {
    let mut coords = GPS_COORDS.lock().unwrap();
    let mut coords_error = GPS_COORDS_ERROR.lock().unwrap();
    let mut coords_last_update = GPS_COORDS_LAST_UPDATE.lock().unwrap();

    let (_, _, c, ce) = match gps::get(false, true) {
        Ok(v) => v,
        Err(err) => {
            debug!("get gps coordinates failed, error: {}", err);
            *coords = None;
            return;
        }
    };

    *coords = Some(c);
    *coords_error = ce;
    *coords_last_update = Some(Utc::now());

    trace!(
        "GPS coordinates sync completed, coords: {:?}, coords_error: {:?}",
        coords, coords_error
    );
}
