use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libconcentratord::signals::Signal;
use libconcentratord::{jitqueue, stats};
use libloragw_sx1302::hal;

use super::gps;
use crate::wrapper;

pub fn stats_loop(
    gateway_id: &[u8],
    get_temperature: bool,
    stats_interval: &Duration,
    stop_receive: Receiver<Signal>,
    mut metadata: HashMap<String, String>,
    queue: Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
) {
    debug!("Starting stats loop, stats_interval: {:?}", stats_interval);

    loop {
        // Instead of a 'stats interval' sleep, we receive from the stop channel with a
        // timeout equal to the 'stats interval'.
        if let Ok(v) = stop_receive.recv_timeout(*stats_interval) {
            debug!("Received stop signal, signal: {}", v);
            break;
        }

        // fetch the current gps coordinates
        let loc = gps::get_coords().map(|v| chirpstack_api::common::Location {
            latitude: v.latitude,
            longitude: v.longitude,
            altitude: v.altitude as f64,
            source: chirpstack_api::common::LocationSource::Gps.into(),
            ..Default::default()
        });

        // fetch the concentrator temperature.
        if get_temperature {
            match hal::get_temperature() {
                Ok(v) => {
                    metadata.insert("concentrator_temp".to_string(), format!("{}", v));
                }
                Err(err) => {
                    metadata.remove(&"concentrator_temp".to_string());
                    error!("Get concentrator temperature error, error: {}", err);
                }
            }
        }

        get_duty_cycle_stats(&queue);

        stats::send_and_reset(gateway_id, loc, &metadata).expect("sending stats failed");
    }

    debug!("Stats loop ended");
}

fn get_duty_cycle_stats(queue: &Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>) {
    let mut queue = queue.lock().unwrap();
    let concentrator_count = hal::get_instcnt().expect("get contentrator count");
    queue.get_duty_cycle_stats(concentrator_count);
}
