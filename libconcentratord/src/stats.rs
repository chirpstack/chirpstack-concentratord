use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

use anyhow::Result;

use super::events;

lazy_static! {
    static ref STATS: Mutex<chirpstack_api::gw::GatewayStats> = Mutex::new(Default::default());
}

pub fn inc_rx_counts(pl: &chirpstack_api::gw::UplinkFrame) {
    let mut stats = STATS.lock().unwrap();
    stats.rx_packets_received_ok += 1;

    if let Some(tx_info) = &pl.tx_info {
        stats
            .rx_packets_per_frequency
            .entry(tx_info.frequency)
            .and_modify(|v| *v += 1)
            .or_insert(1);

        let mut found = false;
        for mod_count in &mut stats.rx_packets_per_modulation {
            if mod_count.modulation == tx_info.modulation {
                mod_count.count += 1;
                found = true;
            }
        }

        if !found {
            stats
                .rx_packets_per_modulation
                .push(chirpstack_api::gw::PerModulationCount {
                    modulation: tx_info.modulation.clone(),
                    count: 1,
                });
        }
    }
}

pub fn inc_tx_counts(tx_info: &chirpstack_api::gw::DownlinkTxInfo) {
    let mut stats = STATS.lock().unwrap();
    stats.tx_packets_emitted += 1;

    stats
        .tx_packets_per_frequency
        .entry(tx_info.frequency)
        .and_modify(|v| *v += 1)
        .or_insert(1);

    let mut found = false;
    for mod_count in &mut stats.tx_packets_per_modulation {
        if mod_count.modulation == tx_info.modulation {
            mod_count.count += 1;
            found = true;
        }
    }

    if !found {
        stats
            .tx_packets_per_modulation
            .push(chirpstack_api::gw::PerModulationCount {
                modulation: tx_info.modulation.clone(),
                count: 1,
            });
    }
}

pub fn inc_tx_status_count(status: chirpstack_api::gw::TxAckStatus) {
    let mut stats = STATS.lock().unwrap();
    stats
        .tx_packets_per_status
        .entry(status.as_str_name().to_string())
        .and_modify(|v| *v += 1)
        .or_insert(1);
}

pub fn inc_rx_packets_received() {
    let mut stats = STATS.lock().unwrap();
    stats.rx_packets_received += 1;
}

pub fn inc_tx_packets_received() {
    let mut stats = STATS.lock().unwrap();
    stats.tx_packets_received += 1;
}

pub fn send_and_reset(
    gateway_id: &[u8],
    location: Option<chirpstack_api::common::Location>,
    duty_cycle_stats: Option<chirpstack_api::gw::DutyCycleStats>,
    metadata: &HashMap<String, String>,
) -> Result<()> {
    let mut stats = STATS.lock().unwrap();

    stats.gateway_id = hex::encode(gateway_id);
    stats.time = Some(prost_types::Timestamp::from(SystemTime::now()));
    stats.location = location;
    stats.duty_cycle_stats = duty_cycle_stats;
    stats.metadata.clone_from(metadata);

    events::send_stats(&stats).unwrap();

    // reset stats
    *stats = Default::default();

    Ok(())
}
