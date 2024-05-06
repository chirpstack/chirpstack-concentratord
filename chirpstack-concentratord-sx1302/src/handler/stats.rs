use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};

use chirpstack_api::gw::DutyCycleStats;
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
) -> Result<()> {
    debug!("Starting stats loop, stats_interval: {:?}", stats_interval);

    loop {
        // Instead of a 'stats interval' sleep, we receive from the stop channel with a
        // timeout equal to the 'stats interval'.
        if let Ok(v) = stop_receive.recv_timeout(*stats_interval) {
            debug!("Received stop signal, signal: {}", v);
            return Ok(());
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
                    metadata.remove("concentrator_temp");
                    error!("Get concentrator temperature error, error: {}", err);
                }
            }
        }

        let dc_stats = get_duty_cycle_stats(&queue)?;
        stats::send_and_reset(gateway_id, loc, dc_stats, &metadata).context("Send stats")?;
    }
}

fn get_duty_cycle_stats(
    queue: &Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
) -> Result<Option<DutyCycleStats>> {
    let mut queue = queue.lock().map_err(|_| anyhow!("Lock queue error"))?;
    let concentrator_count = hal::get_instcnt()?;
    Ok(queue.get_duty_cycle_stats(concentrator_count))
}
