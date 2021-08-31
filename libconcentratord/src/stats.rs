use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use super::events;

lazy_static! {
    static ref STATS: Mutex<chirpstack_api::gw::GatewayStats> = Mutex::new(Default::default());
}

pub fn inc_rx_counts(pl: &chirpstack_api::gw::UplinkFrame) {
    let mut stats = STATS.lock().unwrap();
    stats.rx_packets_received_ok += 1;

    match &pl.tx_info {
        Some(tx_info) => {
            stats
                .rx_packets_per_frequency
                .entry(tx_info.frequency)
                .and_modify(|v| *v += 1)
                .or_insert(1);

            match &tx_info.modulation_info {
                Some(mod_info) => {
                    let modulation = Some(chirpstack_api::gw::Modulation {
                        parameters: Some(match &mod_info {
                        chirpstack_api::gw::uplink_tx_info::ModulationInfo::LoraModulationInfo(
                            v,
                        ) => chirpstack_api::gw::modulation::Parameters::Lora(v.clone()),
                        chirpstack_api::gw::uplink_tx_info::ModulationInfo::FskModulationInfo(
                            v,
                        ) => chirpstack_api::gw::modulation::Parameters::Fsk(v.clone()),
                        chirpstack_api::gw::uplink_tx_info::ModulationInfo::LrFhssModulationInfo(
                            v,
                        ) => chirpstack_api::gw::modulation::Parameters::LrFhss(v.clone()),
                    })});

                    let mut found = false;
                    for mod_count in &mut stats.rx_packets_per_modulation {
                        if mod_count.modulation == modulation {
                            mod_count.count += 1;
                            found = true;
                        }
                    }

                    if !found {
                        stats.rx_packets_per_modulation.push(
                            chirpstack_api::gw::PerModulationCount {
                                modulation: modulation,
                                count: 1,
                            },
                        );
                    }
                }
                None => {}
            };
        }
        None => {}
    };
}

pub fn inc_tx_counts(tx_info: &chirpstack_api::gw::DownlinkTxInfo) {
    let mut stats = STATS.lock().unwrap();
    stats.tx_packets_emitted += 1;

    stats
        .tx_packets_per_frequency
        .entry(tx_info.frequency)
        .and_modify(|v| *v += 1)
        .or_insert(1);

    match &tx_info.modulation_info {
        Some(mod_info) => {
            let modulation = Some(chirpstack_api::gw::Modulation {
                parameters: Some(match &mod_info {
                    chirpstack_api::gw::downlink_tx_info::ModulationInfo::LoraModulationInfo(v) => {
                        chirpstack_api::gw::modulation::Parameters::Lora(v.clone())
                    }
                    chirpstack_api::gw::downlink_tx_info::ModulationInfo::FskModulationInfo(v) => {
                        chirpstack_api::gw::modulation::Parameters::Fsk(v.clone())
                    }
                }),
            });

            let mut found = false;
            for mod_count in &mut stats.tx_packets_per_modulation {
                if mod_count.modulation == modulation {
                    mod_count.count += 1;
                    found = true;
                }
            }

            if !found {
                stats
                    .tx_packets_per_modulation
                    .push(chirpstack_api::gw::PerModulationCount {
                        modulation: modulation,
                        count: 1,
                    });
            }
        }
        None => {}
    };
}

pub fn inc_tx_status_count(status: chirpstack_api::gw::TxAckStatus) {
    let s = match status {
        chirpstack_api::gw::TxAckStatus::Ignored => "IGNORED",
        chirpstack_api::gw::TxAckStatus::Ok => "OK",
        chirpstack_api::gw::TxAckStatus::TooLate => "TOO_LATE",
        chirpstack_api::gw::TxAckStatus::TooEarly => "TOO_EARLY",
        chirpstack_api::gw::TxAckStatus::CollisionPacket => "COLLISION_PACKET",
        chirpstack_api::gw::TxAckStatus::CollisionBeacon => "COLLISION_BEACON",
        chirpstack_api::gw::TxAckStatus::TxFreq => "TX_FREQ",
        chirpstack_api::gw::TxAckStatus::TxPower => "TX_POWER",
        chirpstack_api::gw::TxAckStatus::GpsUnlocked => "GPS_UNLOCKED",
        chirpstack_api::gw::TxAckStatus::QueueFull => "QUEUE_FULL",
        chirpstack_api::gw::TxAckStatus::InternalError => "InternalError",
    }
    .to_string();

    let mut stats = STATS.lock().unwrap();
    stats
        .tx_packets_per_status
        .entry(s)
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
    metadata: &HashMap<String, String>,
) -> Result<(), String> {
    let mut stats = STATS.lock().unwrap();

    let stats_id = Uuid::new_v4();
    let now_since_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    stats.gateway_id = gateway_id.to_vec();
    stats.stats_id = stats_id.as_bytes().to_vec();
    stats.time = Some(prost_types::Timestamp {
        seconds: now_since_unix.as_secs() as i64,
        nanos: now_since_unix.subsec_nanos() as i32,
    });
    stats.location = location;
    stats.meta_data = metadata.clone();

    events::send_stats(&stats, &stats_id).unwrap();

    // reset stats
    *stats = Default::default();

    return Ok(());
}
