use std::sync::mpsc::Receiver;
use std::time::Duration;

use libconcentratord::signals::Signal;
use libconcentratord::stats;

use super::gps;

pub fn stats_loop(gateway_id: &[u8], stats_interval: &Duration, stop_receive: Receiver<Signal>) {
    debug!("Starting stats loop, stats_interval: {:?}", stats_interval);

    loop {
        // Instead of a 'stats interval' sleep, we receive from the stop channel with a
        // timeout equal to the 'stats interval'.
        match stop_receive.recv_timeout(*stats_interval) {
            Ok(v) => {
                debug!("Received stop signal, signal: {}", v);
                return;
            }
            _ => {}
        };

        // fetch the current gps coordinates
        let loc = match gps::get_coords() {
            Ok(v) => Some({
                let mut loc = chirpstack_api::common::Location {
                    latitude: v.latitude,
                    longitude: v.longitude,
                    altitude: v.altitude as f64,
                    ..Default::default()
                };

                loc.set_source(chirpstack_api::common::LocationSource::Gps);
                loc
            }),
            Err(err) => {
                debug!("Get gps coordinates error, error: {}", err);
                None
            }
        };

        stats::send_and_reset(gateway_id, loc).expect("sending stats failed");
    }
}
