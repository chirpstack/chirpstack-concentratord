use std::time::{Duration, UNIX_EPOCH};

use libconcentratord::jitqueue;
use libloragw_sx1302::hal;
use protobuf::well_known_types;
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
    let mut tx_info: chirpstack_api::gw::UplinkTXInfo = Default::default();
    tx_info.set_frequency(packet.freq_hz);

    match packet.modulation {
        hal::Modulation::LoRa => {
            let mut mod_info: chirpstack_api::gw::LoRaModulationInfo = Default::default();
            mod_info.set_bandwidth(packet.bandwidth);
            mod_info.set_spreading_factor(match packet.datarate {
                hal::DataRate::SF5 => 5,
                hal::DataRate::SF6 => 6,
                hal::DataRate::SF7 => 7,
                hal::DataRate::SF8 => 8,
                hal::DataRate::SF9 => 9,
                hal::DataRate::SF10 => 10,
                hal::DataRate::SF11 => 11,
                hal::DataRate::SF12 => 12,
                _ => return Err("unexpected spreading-factor".to_string()),
            });
            mod_info.set_code_rate(match packet.coderate {
                hal::CodeRate::LoRa4_5 => "4/5".to_string(),
                hal::CodeRate::LoRa4_6 => "5/6".to_string(),
                hal::CodeRate::LoRa4_7 => "5/7".to_string(),
                hal::CodeRate::LoRa4_8 => "5/8".to_string(),
                hal::CodeRate::Undefined => "".to_string(),
            });

            tx_info.set_modulation(chirpstack_api::common::Modulation::LORA);
            tx_info.set_lora_modulation_info(mod_info);
        }
        hal::Modulation::FSK => {
            let mut mod_info: chirpstack_api::gw::FSKModulationInfo = Default::default();
            mod_info.set_datarate(match packet.datarate {
                hal::DataRate::FSK(v) => v * 1000,
                _ => return Err("unexpected datarate".to_string()),
            });

            tx_info.set_modulation(chirpstack_api::common::Modulation::FSK);
            tx_info.set_fsk_modulation_info(mod_info);
        }
        hal::Modulation::Undefined => {
            return Err("undefined modulation".to_string());
        }
    }

    // rx info
    let mut rx_info: chirpstack_api::gw::UplinkRXInfo = Default::default();
    let uplink_id = Uuid::new_v4();

    rx_info.set_uplink_id(uplink_id.as_bytes().to_vec());
    rx_info.set_context(packet.count_us.to_be_bytes().to_vec());
    rx_info.set_gateway_id(gateway_id.to_vec());
    rx_info.set_rssi(packet.rssis as i32);
    rx_info.set_lora_snr(packet.snr as f64);
    rx_info.set_channel(packet.if_chain as u32);
    rx_info.set_rf_chain(packet.rf_chain as u32);
    rx_info.set_board(0);
    rx_info.set_antenna(0);
    rx_info.set_crc_status(match packet.status {
        hal::CRC::Undefined => chirpstack_api::gw::CRCStatus::NO_CRC,
        hal::CRC::NoCRC => chirpstack_api::gw::CRCStatus::NO_CRC,
        hal::CRC::BadCRC => chirpstack_api::gw::CRCStatus::BAD_CRC,
        hal::CRC::CRCOk => chirpstack_api::gw::CRCStatus::CRC_OK,
    });
    match gps::cnt2time(packet.count_us) {
        Ok(v) => {
            let v = v.duration_since(UNIX_EPOCH).unwrap();
            let mut proto_ts = well_known_types::Timestamp::new();
            proto_ts.set_seconds(v.as_secs() as i64);
            proto_ts.set_nanos(v.subsec_nanos() as i32);
            rx_info.set_time(proto_ts);
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
            let mut proto_dur = well_known_types::Duration::new();
            proto_dur.set_seconds(v.as_secs() as i64);
            proto_dur.set_nanos(v.subsec_nanos() as i32);
            rx_info.set_time_since_gps_epoch(proto_dur);
        }
        Err(err) => {
            debug!(
                "Could not get GPS epoch, uplink_id: {}, error: {}",
                uplink_id, err
            );
        }
    }
    match gps::get_coords() {
        Ok(v) => {
            let mut proto_loc = chirpstack_api::common::Location::new();
            proto_loc.set_source(chirpstack_api::common::LocationSource::GPS);
            proto_loc.set_latitude(v.latitude);
            proto_loc.set_longitude(v.longitude);
            proto_loc.set_altitude(v.altitude as f64);

            rx_info.set_location(proto_loc);
        }
        Err(err) => {
            debug!(
                "Could not get GPS coordinates, uplink_id: {}, error: {}",
                uplink_id, err
            );
        }
    }

    let mut pb: chirpstack_api::gw::UplinkFrame = Default::default();
    pb.set_phy_payload(packet.payload[..packet.size as usize].to_vec());
    pb.set_tx_info(tx_info);
    pb.set_rx_info(rx_info);

    return Ok(pb);
}

pub fn downlink_from_proto(
    df: &chirpstack_api::gw::DownlinkFrame,
) -> Result<hal::TxPacket, String> {
    let mut data: [u8; 256] = [0; 256];
    let mut data_slice = df.phy_payload.clone();
    data_slice.resize(data.len(), 0);
    data.copy_from_slice(&data_slice);

    let packet = hal::TxPacket {
        freq_hz: df.get_tx_info().frequency,
        tx_mode: match df.get_tx_info().get_timing() {
            chirpstack_api::gw::DownlinkTiming::DELAY => hal::TxMode::Timestamped,
            chirpstack_api::gw::DownlinkTiming::GPS_EPOCH => hal::TxMode::Timestamped, // the epoch timestamp is converted to count_us below
            chirpstack_api::gw::DownlinkTiming::IMMEDIATELY => hal::TxMode::Immediate,
        },
        count_us: match df.get_tx_info().get_timing() {
            chirpstack_api::gw::DownlinkTiming::DELAY => {
                let ctx = df.get_tx_info().get_context();
                let delay = df.get_tx_info().get_delay_timing_info().get_delay();
                if ctx.len() != 4 {
                    return Err("context must be exactly 4 bytes".to_string());
                }

                let mut array = [0; 4];
                array.copy_from_slice(ctx);
                u32::from_be_bytes(array).wrapping_add(
                    (Duration::from_secs(delay.get_seconds() as u64)
                        + Duration::from_nanos(delay.get_nanos() as u64))
                    .as_micros() as u32,
                )
            }
            chirpstack_api::gw::DownlinkTiming::GPS_EPOCH => {
                let gps_epoch_proto = df
                    .get_tx_info()
                    .get_gps_epoch_timing_info()
                    .get_time_since_gps_epoch();

                let gps_epoch = Duration::from_secs(gps_epoch_proto.get_seconds() as u64)
                    + Duration::from_nanos(gps_epoch_proto.get_nanos() as u64);

                match gps::epoch2cnt(&gps_epoch) {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                }
            }
            _ => 0,
        },
        rf_chain: 0,
        rf_power: df.get_tx_info().get_power() as i8,
        modulation: match df.get_tx_info().get_modulation() {
            chirpstack_api::common::Modulation::LORA => hal::Modulation::LoRa,
            chirpstack_api::common::Modulation::FSK => hal::Modulation::FSK,
        },
        bandwidth: match df.get_tx_info().get_modulation() {
            chirpstack_api::common::Modulation::LORA => {
                df.get_tx_info().get_lora_modulation_info().get_bandwidth()
            }
            _ => 0,
        },
        datarate: match df.get_tx_info().get_modulation() {
            chirpstack_api::common::Modulation::LORA => {
                match df
                    .get_tx_info()
                    .get_lora_modulation_info()
                    .get_spreading_factor()
                {
                    5 => hal::DataRate::SF5,
                    6 => hal::DataRate::SF6,
                    7 => hal::DataRate::SF7,
                    8 => hal::DataRate::SF8,
                    9 => hal::DataRate::SF9,
                    10 => hal::DataRate::SF10,
                    11 => hal::DataRate::SF11,
                    12 => hal::DataRate::SF12,
                    _ => return Err("unexpected spreading-factor".to_string()),
                }
            }
            chirpstack_api::common::Modulation::FSK => {
                hal::DataRate::FSK(df.get_tx_info().get_fsk_modulation_info().get_datarate())
            }
        },
        coderate: match df.get_tx_info().get_modulation() {
            chirpstack_api::common::Modulation::FSK => hal::CodeRate::Undefined,
            chirpstack_api::common::Modulation::LORA => {
                match df
                    .get_tx_info()
                    .get_lora_modulation_info()
                    .get_code_rate()
                    .as_ref()
                {
                    "4/5" => hal::CodeRate::LoRa4_5,
                    "4/6" => hal::CodeRate::LoRa4_6,
                    "4/7" => hal::CodeRate::LoRa4_7,
                    "4/8" => hal::CodeRate::LoRa4_8,
                    _ => hal::CodeRate::Undefined,
                }
            }
        },
        invert_pol: df
            .get_tx_info()
            .get_lora_modulation_info()
            .get_polarization_inversion(),
        f_dev: match df.get_tx_info().get_modulation() {
            chirpstack_api::common::Modulation::FSK => {
                (df.get_tx_info()
                    .get_fsk_modulation_info()
                    .get_frequency_deviation()
                    / 1000) as u8
            }
            _ => 0,
        },
        freq_offset: 0,
        preamble: 0,
        no_crc: false,
        no_header: false,
        size: df.get_phy_payload().len() as u16,
        payload: data,
    };

    return Ok(packet);
}
