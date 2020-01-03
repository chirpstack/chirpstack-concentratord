use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use protobuf::well_known_types;
use uuid::Uuid;

use super::events;

lazy_static! {
    static ref STATS: Mutex<chirpstack_api::gw::GatewayStats> = Mutex::new(Default::default());
}

pub fn inc_rx_packets_received() {
    let mut stats = STATS.lock().unwrap();
    stats.rx_packets_received += 1;
}

pub fn inc_rx_packets_received_ok() {
    let mut stats = STATS.lock().unwrap();
    stats.rx_packets_received_ok += 1;
}

pub fn inc_tx_packets_received() {
    let mut stats = STATS.lock().unwrap();
    stats.tx_packets_received += 1;
}

pub fn inc_tx_packets_emitted() {
    let mut stats = STATS.lock().unwrap();
    stats.tx_packets_emitted += 1;
}

pub fn send_and_reset(
    gateway_id: &[u8],
    location: Option<chirpstack_api::common::Location>,
) -> Result<(), String> {
    let mut stats = STATS.lock().unwrap();

    let stats_id = Uuid::new_v4();
    let now_since_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    stats.set_gateway_id(gateway_id.to_vec());
    stats.set_stats_id(stats_id.as_bytes().to_vec());
    stats.set_time({
        let mut ts = well_known_types::Timestamp::default();

        ts.set_seconds(now_since_unix.as_secs() as i64);
        ts.set_nanos(now_since_unix.subsec_nanos() as i32);
        ts
    });

    match location {
        Some(v) => stats.set_location(v),
        None => {}
    }

    events::send_stats(&stats, &stats_id).unwrap();

    // reset stats
    *stats = Default::default();

    return Ok(());
}
