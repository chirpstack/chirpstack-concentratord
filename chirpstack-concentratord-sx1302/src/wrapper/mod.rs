use std::time::{Duration, UNIX_EPOCH};

use libconcentratord::jitqueue;
use libloragw_sx1302::hal;
use uuid::Uuid;

use super::handler::gps;

#[derive(Copy, Clone)]
pub struct TxPacket(hal::TxPacket, Uuid);

impl TxPacket {
    pub fn new(id: Uuid, tx_packet: hal::TxPacket) -> TxPacket {
        TxPacket(tx_packet, id)
    }

    pub fn tx_packet(&self) -> hal::TxPacket {
        self.0
    }
}

impl jitqueue::TxPacket for TxPacket {
    fn get_time_on_air(&self) -> Result<Duration, String> {
        hal::time_on_air(&self.0)
    }

    fn get_tx_mode(&self) -> jitqueue::TxMode {
        match self.0.tx_mode {
            hal::TxMode::Timestamped => jitqueue::TxMode::Timestamped,
            hal::TxMode::OnGPS => jitqueue::TxMode::OnGPS,
            hal::TxMode::Immediate => jitqueue::TxMode::Immediate,
        }
    }
    fn set_tx_mode(&mut self, tx_mode: jitqueue::TxMode) {
        self.0.tx_mode = match tx_mode {
            jitqueue::TxMode::Timestamped => hal::TxMode::Timestamped,
            jitqueue::TxMode::OnGPS => hal::TxMode::OnGPS,
            jitqueue::TxMode::Immediate => hal::TxMode::Immediate,
        };
    }
    fn get_count_us(&self) -> u32 {
        self.0.count_us
    }
    fn set_count_us(&mut self, count_us: u32) {
        self.0.count_us = count_us;
    }

    fn get_id(&self) -> String {
        self.1.to_string()
    }
}

pub fn uplink_to_proto(
    gateway_id: &[u8],
    packet: &hal::RxPacket,
) -> Result<chirpstack_api::gw::UplinkFrame, String> {
    // tx info
    let mut tx_info: chirpstack_api::gw::UplinkTxInfo = Default::default();
    tx_info.frequency = packet.freq_hz;

    match packet.modulation {
        hal::Modulation::LoRa => {
            let mut mod_info: chirpstack_api::gw::LoRaModulationInfo = Default::default();
            mod_info.bandwidth = packet.bandwidth;
            mod_info.spreading_factor = match packet.datarate {
                hal::DataRate::SF5 => 5,
                hal::DataRate::SF6 => 6,
                hal::DataRate::SF7 => 7,
                hal::DataRate::SF8 => 8,
                hal::DataRate::SF9 => 9,
                hal::DataRate::SF10 => 10,
                hal::DataRate::SF11 => 11,
                hal::DataRate::SF12 => 12,
                _ => return Err("unexpected spreading-factor".to_string()),
            };
            mod_info.code_rate = match packet.coderate {
                hal::CodeRate::LoRa4_5 => "4/5".to_string(),
                hal::CodeRate::LoRa4_6 => "5/6".to_string(),
                hal::CodeRate::LoRa4_7 => "5/7".to_string(),
                hal::CodeRate::LoRa4_8 => "5/8".to_string(),
                hal::CodeRate::Undefined => "".to_string(),
            };

            tx_info.set_modulation(chirpstack_api::common::Modulation::Lora);
            tx_info.modulation_info = Some(
                chirpstack_api::gw::uplink_tx_info::ModulationInfo::LoraModulationInfo(mod_info),
            );
        }
        hal::Modulation::FSK => {
            let mut mod_info: chirpstack_api::gw::FskModulationInfo = Default::default();
            mod_info.datarate = match packet.datarate {
                hal::DataRate::FSK(v) => v * 1000,
                _ => return Err("unexpected datarate".to_string()),
            };

            tx_info.set_modulation(chirpstack_api::common::Modulation::Fsk);
            tx_info.modulation_info = Some(
                chirpstack_api::gw::uplink_tx_info::ModulationInfo::FskModulationInfo(mod_info),
            );
        }
        hal::Modulation::Undefined => {
            return Err("undefined modulation".to_string());
        }
    }

    // rx info
    let mut rx_info: chirpstack_api::gw::UplinkRxInfo = Default::default();
    let uplink_id = Uuid::new_v4();

    rx_info.uplink_id = uplink_id.as_bytes().to_vec();
    rx_info.context = packet.count_us.to_be_bytes().to_vec();
    rx_info.gateway_id = gateway_id.to_vec();
    rx_info.rssi = packet.rssis as i32;
    rx_info.lora_snr = packet.snr as f64;
    rx_info.channel = packet.if_chain as u32;
    rx_info.rf_chain = packet.rf_chain as u32;
    rx_info.board = 0;
    rx_info.antenna = 0;
    rx_info.set_crc_status(match packet.status {
        hal::CRC::Undefined => chirpstack_api::gw::CrcStatus::NoCrc,
        hal::CRC::NoCRC => chirpstack_api::gw::CrcStatus::NoCrc,
        hal::CRC::BadCRC => chirpstack_api::gw::CrcStatus::BadCrc,
        hal::CRC::CRCOk => chirpstack_api::gw::CrcStatus::CrcOk,
    });
    match gps::cnt2time(packet.count_us) {
        Ok(v) => {
            let v = v.duration_since(UNIX_EPOCH).unwrap();

            rx_info.time = Some(prost_types::Timestamp {
                seconds: v.as_secs() as i64,
                nanos: v.subsec_nanos() as i32,
            });
        }
        Err(err) => {
            debug!(
                "Could not get GPS time, uplink_id: {}, error: {}",
                uplink_id, err
            );
        }
    };
    match gps::cnt2epoch(packet.count_us) {
        Ok(v) => {
            rx_info.time_since_gps_epoch = Some(prost_types::Duration {
                seconds: v.as_secs() as i64,
                nanos: v.subsec_nanos() as i32,
            });
        }
        Err(err) => {
            debug!(
                "Could not get GPS epoch, uplink_id: {}, error: {}",
                uplink_id, err
            );
        }
    }
    match gps::get_coords() {
        Some(v) => {
            let mut proto_loc = chirpstack_api::common::Location {
                latitude: v.latitude,
                longitude: v.longitude,
                altitude: v.altitude as f64,
                ..Default::default()
            };
            proto_loc.set_source(chirpstack_api::common::LocationSource::Gps);

            rx_info.location = Some(proto_loc);
        }
        None => {}
    }

    let mut pb: chirpstack_api::gw::UplinkFrame = Default::default();

    pb.phy_payload = packet.payload[..packet.size as usize].to_vec();
    pb.tx_info = Some(tx_info);
    pb.rx_info = Some(rx_info);

    return Ok(pb);
}

pub fn downlink_from_proto(
    df: &chirpstack_api::gw::DownlinkFrameItem,
) -> Result<hal::TxPacket, String> {
    let mut data: [u8; 256] = [0; 256];
    let mut data_slice = df.phy_payload.clone();
    data_slice.resize(data.len(), 0);
    data.copy_from_slice(&data_slice);

    let tx_info = match df.tx_info.as_ref() {
        Some(v) => v,
        None => return Err("tx_info must not be blank".to_string()),
    };

    let mut packet = hal::TxPacket {
        freq_hz: tx_info.frequency,
        tx_mode: match tx_info.timing() {
            chirpstack_api::gw::DownlinkTiming::Delay => hal::TxMode::Timestamped,
            chirpstack_api::gw::DownlinkTiming::GpsEpoch => hal::TxMode::Timestamped, // the epoch timestamp is converted to count_us below
            chirpstack_api::gw::DownlinkTiming::Immediately => hal::TxMode::Immediate,
        },
        modulation: match tx_info.modulation() {
            chirpstack_api::common::Modulation::Lora => hal::Modulation::LoRa,
            chirpstack_api::common::Modulation::Fsk => hal::Modulation::FSK,
            chirpstack_api::common::Modulation::LrFhss => {
                return Err("lr-fhss modulation is not supported".to_string());
            }
        },
        rf_chain: 0,
        rf_power: tx_info.power as i8,
        freq_offset: 0,
        preamble: 0,
        no_crc: false,
        no_header: false,
        size: df.phy_payload.len() as u16,
        payload: data,
        ..Default::default()
    };

    match &tx_info.timing_info {
        Some(chirpstack_api::gw::downlink_tx_info::TimingInfo::DelayTimingInfo(v)) => {
            let ctx = &tx_info.context;
            if ctx.len() != 4 {
                return Err("context must be exactly 4 bytes".to_string());
            }

            match v.delay.as_ref() {
                Some(v) => {
                    let mut array = [0; 4];
                    array.copy_from_slice(ctx);
                    packet.count_us = u32::from_be_bytes(array).wrapping_add(
                        (Duration::from_secs(v.seconds as u64)
                            + Duration::from_nanos(v.nanos as u64))
                        .as_micros() as u32,
                    )
                }
                None => {
                    return Err("delay must not be nil".to_string());
                }
            }
        }
        Some(chirpstack_api::gw::downlink_tx_info::TimingInfo::GpsEpochTimingInfo(v)) => {
            match v.time_since_gps_epoch.as_ref() {
                Some(v) => {
                    let gps_epoch = Duration::from_secs(v.seconds as u64)
                        + Duration::from_nanos(v.nanos as u64);

                    match gps::epoch2cnt(&gps_epoch) {
                        Ok(v) => {
                            packet.count_us = v;
                        }
                        Err(err) => return Err(err),
                    }
                }
                None => {
                    return Err("time_since_gps_epoch must not be nil".to_string());
                }
            }
        }
        Some(chirpstack_api::gw::downlink_tx_info::TimingInfo::ImmediatelyTimingInfo(_)) => {}
        None => {}
    };

    match &tx_info.modulation_info {
        Some(chirpstack_api::gw::downlink_tx_info::ModulationInfo::LoraModulationInfo(v)) => {
            packet.bandwidth = v.bandwidth;
            packet.datarate = match v.spreading_factor {
                5 => hal::DataRate::SF5,
                6 => hal::DataRate::SF6,
                7 => hal::DataRate::SF7,
                8 => hal::DataRate::SF8,
                9 => hal::DataRate::SF9,
                10 => hal::DataRate::SF10,
                11 => hal::DataRate::SF11,
                12 => hal::DataRate::SF12,
                _ => return Err("unexpected spreading-factor".to_string()),
            };
            packet.coderate = match v.code_rate.as_ref() {
                "4/5" => hal::CodeRate::LoRa4_5,
                "4/6" => hal::CodeRate::LoRa4_6,
                "4/7" => hal::CodeRate::LoRa4_7,
                "4/8" => hal::CodeRate::LoRa4_8,
                _ => hal::CodeRate::Undefined,
            };
            packet.invert_pol = v.polarization_inversion;
        }
        Some(chirpstack_api::gw::downlink_tx_info::ModulationInfo::FskModulationInfo(v)) => {
            packet.datarate = hal::DataRate::FSK(v.datarate);
            packet.f_dev = (v.frequency_deviation / 1000) as u8;
        }
        None => {}
    }

    return Ok(packet);
}

pub fn downlink_to_tx_info_proto(
    packet: &hal::TxPacket,
) -> Result<chirpstack_api::gw::DownlinkTxInfo, String> {
    let mut tx_info: chirpstack_api::gw::DownlinkTxInfo = Default::default();
    tx_info.frequency = packet.freq_hz;

    match packet.modulation {
        hal::Modulation::LoRa => {
            let mut mod_info: chirpstack_api::gw::LoRaModulationInfo = Default::default();
            mod_info.bandwidth = packet.bandwidth;
            mod_info.spreading_factor = match packet.datarate {
                hal::DataRate::SF5 => 5,
                hal::DataRate::SF6 => 6,
                hal::DataRate::SF7 => 7,
                hal::DataRate::SF8 => 8,
                hal::DataRate::SF9 => 9,
                hal::DataRate::SF10 => 10,
                hal::DataRate::SF11 => 11,
                hal::DataRate::SF12 => 12,
                _ => {
                    return Err("unexpected spreading-factor".to_string());
                }
            };
            mod_info.code_rate = match packet.coderate {
                hal::CodeRate::LoRa4_5 => "4/5".to_string(),
                hal::CodeRate::LoRa4_6 => "4/6".to_string(),
                hal::CodeRate::LoRa4_7 => "4/7".to_string(),
                hal::CodeRate::LoRa4_8 => "4/8".to_string(),
                hal::CodeRate::Undefined => "".to_string(),
            };

            tx_info.set_modulation(chirpstack_api::common::Modulation::Lora);
            tx_info.modulation_info = Some(
                chirpstack_api::gw::downlink_tx_info::ModulationInfo::LoraModulationInfo(mod_info),
            );
        }
        hal::Modulation::FSK => {
            let mut mod_info: chirpstack_api::gw::FskModulationInfo = Default::default();
            mod_info.datarate = match packet.datarate {
                hal::DataRate::FSK(v) => v * 1000,
                _ => return Err("unexpected datarate".to_string()),
            };

            tx_info.set_modulation(chirpstack_api::common::Modulation::Fsk);
            tx_info.modulation_info = Some(
                chirpstack_api::gw::downlink_tx_info::ModulationInfo::FskModulationInfo(mod_info),
            );
        }
        hal::Modulation::Undefined => {
            return Err("undefined modulation".to_string());
        }
    }

    Ok(tx_info)
}
