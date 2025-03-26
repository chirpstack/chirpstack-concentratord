use std::sync::{LazyLock, Mutex};

use libloragw_2g4::gps;

static STATIC_GPS_COORDS: LazyLock<Mutex<Option<gps::Coordinates>>> =
    LazyLock::new(|| Mutex::new(None));

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

pub fn get_coords() -> Option<gps::Coordinates> {
    let static_gps_coords = STATIC_GPS_COORDS.lock().unwrap();
    *static_gps_coords
}
