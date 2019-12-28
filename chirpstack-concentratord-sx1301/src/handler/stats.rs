use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use protobuf::{well_known_types, Message};
use uuid::Uuid;

use super::gps;

lazy_static! {
    static ref STATS: Mutex<Stats> = Mutex::new(Default::default());
}

#[derive(Default)]
pub struct Stats {
    rx_packets_received: u32,
    rx_packets_received_ok: u32,
    tx_packets_received: u32,
    tx_packets_emitted: u32,
}

pub fn stats_loop(gateway_id: &[u8], stats_interval: &Duration, publisher: zmq::Socket) {
    debug!("Starting stats loop, stats_interval: {:?}", stats_interval);

    loop {
        thread::sleep(*stats_interval);

        // New scope so that mutex is released after completion.
        {
            let mut stats = STATS.lock().unwrap();

            let stats_id = Uuid::new_v4();
            let now_since_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

            let mut stats_proto = chirpstack_api::gw::GatewayStats::default();
            stats_proto.set_rx_packets_received(stats.rx_packets_received);
            stats_proto.set_rx_packets_received_ok(stats.rx_packets_received_ok);
            stats_proto.set_tx_packets_received(stats.tx_packets_received);
            stats_proto.set_tx_packets_emitted(stats.tx_packets_emitted);
            stats_proto.set_gateway_id(gateway_id.to_vec());
            stats_proto.set_stats_id(stats_id.as_bytes().to_vec());
            stats_proto.set_time({
                let mut ts = well_known_types::Timestamp::default();

                ts.set_seconds(now_since_unix.as_secs() as i64);
                ts.set_nanos(now_since_unix.subsec_nanos() as i32);
                ts
            });

            match gps::get_coords() {
                Ok(v) => {
                    let mut loc = chirpstack_api::common::Location::default();
                    loc.set_latitude(v.latitude);
                    loc.set_longitude(v.longitude);
                    loc.set_altitude(v.altitude as f64);
                    loc.set_source(chirpstack_api::common::LocationSource::GPS);
                    stats_proto.set_location(loc);
                }
                Err(err) => {
                    debug!("Get gps coordinates error, error: {}", err);
                }
            }

            info!("Publishing stats event, stats_id: {}, rx_received: {}, rx_received_ok: {}, tx_received: {}, tx_emitted: {}", stats_id, stats.rx_packets_received, stats.rx_packets_received_ok, stats.tx_packets_received, stats.tx_packets_emitted);
            let proto_bytes = stats_proto.write_to_bytes().unwrap();
            publisher.send("stats", zmq::SNDMORE).unwrap();
            publisher.send(proto_bytes, 0).unwrap();

            // reset stats
            *stats = Default::default();
        }
    }
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
