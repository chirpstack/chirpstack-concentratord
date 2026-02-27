use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use std::time::Duration;

use anyhow::Result;

use chirpstack_api::gw::DutyCycleStats;
use libconcentratord::signals::Signal;
use libconcentratord::{gnss, jitqueue, stats};

use super::timersync;
use crate::wrapper;

pub fn stats_loop(
    gateway_id: &[u8],
    stats_interval: &Duration,
    stop_receive: Receiver<Signal>,
    metadata: &HashMap<String, String>,
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
        let loc = gnss::get_location(timersync::get_concentrator_count()).map(|v| {
            chirpstack_api::common::Location {
                latitude: v.lat,
                longitude: v.lon,
                altitude: v.alt.into(),
                source: chirpstack_api::common::LocationSource::Gps.into(),
                ..Default::default()
            }
        });

        let dc_stats = get_duty_cycle_stats(&queue)?;
        stats::send_and_reset(gateway_id, loc, dc_stats, metadata).expect("sending stats failed");
    }
}

fn get_duty_cycle_stats(
    queue: &Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
) -> Result<Option<DutyCycleStats>> {
    let mut queue = queue.lock().map_err(|_| anyhow!("Lock queue error"))?;
    let concentrator_count = timersync::get_concentrator_count();
    Ok(queue.get_duty_cycle_stats(concentrator_count))
}
