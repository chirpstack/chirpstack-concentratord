use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use chirpstack_api::{gw, prost_types};
use libconcentratord::jitqueue;
use libloragw_sx1302::hal;
use rand::Rng;

use super::handler::gps;

#[derive(Copy, Clone)]
pub struct TxPacket(hal::TxPacket, u32);

impl TxPacket {
    pub fn new(id: u32, tx_packet: hal::TxPacket) -> TxPacket {
        TxPacket(tx_packet, id)
    }

    pub fn tx_packet(&self) -> hal::TxPacket {
        self.0
    }
}

impl jitqueue::TxPacket for TxPacket {
    fn get_time_on_air(&self) -> Result<Duration> {
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

    fn get_id(&self) -> u32 {
        self.1
    }

    fn get_frequency(&self) -> u32 {
        self.0.freq_hz
    }

    fn get_tx_power(&self) -> i8 {
        self.0.rf_power
    }
}

pub fn uplink_to_proto(
    gateway_id: &[u8],
    packet: &hal::RxPacket,
    time_fallback: bool,
) -> Result<gw::UplinkFrame> {
    let mut rng = rand::rng();
    let uplink_id: u32 = rng.random();

    let time_since_gps_epoch = match gps::cnt2epoch(packet.count_us) {
        Ok(v) => Some(prost_types::Duration {
            seconds: v.as_secs() as i64,
            nanos: v.subsec_nanos() as i32,
        }),
        Err(err) => {
            debug!(
                "Could not get GPS epoch, uplink_id: {}, error: {}",
                uplink_id, err
            );
            None
        }
    };

    Ok(gw::UplinkFrame {
        phy_payload: packet.payload[..packet.size as usize].to_vec(),
        tx_info: Some(gw::UplinkTxInfo {
            frequency: packet.freq_hz,
            modulation: Some(gw::Modulation {
                parameters: match packet.modulation {
                    hal::Modulation::LoRa => {
                        Some(gw::modulation::Parameters::Lora(gw::LoraModulationInfo {
                            bandwidth: packet.bandwidth,
                            spreading_factor: match packet.datarate {
                                hal::DataRate::SF5 => 5,
                                hal::DataRate::SF6 => 6,
                                hal::DataRate::SF7 => 7,
                                hal::DataRate::SF8 => 8,
                                hal::DataRate::SF9 => 9,
                                hal::DataRate::SF10 => 10,
                                hal::DataRate::SF11 => 11,
                                hal::DataRate::SF12 => 12,
                                _ => return Err(anyhow!("unexpected spreading-factor")),
                            },
                            code_rate: match packet.coderate {
                                hal::CodeRate::LoRa4_5 => gw::CodeRate::Cr45,
                                hal::CodeRate::LoRa4_6 => gw::CodeRate::Cr46,
                                hal::CodeRate::LoRa4_7 => gw::CodeRate::Cr47,
                                hal::CodeRate::LoRa4_8 => gw::CodeRate::Cr48,
                                hal::CodeRate::Undefined => gw::CodeRate::CrUndefined,
                            }
                            .into(),
                            ..Default::default()
                        }))
                    }
                    hal::Modulation::FSK => {
                        Some(gw::modulation::Parameters::Fsk(gw::FskModulationInfo {
                            datarate: match packet.datarate {
                                hal::DataRate::FSK(v) => v,
                                _ => return Err(anyhow!("unexpected datarate")),
                            },
                            ..Default::default()
                        }))
                    }
                    hal::Modulation::Undefined => None,
                },
            }),
        }),
        rx_info: Some(gw::UplinkRxInfo {
            uplink_id,
            context: packet.count_us.to_be_bytes().to_vec(),
            gateway_id: hex::encode(gateway_id),
            rssi: packet.rssis as i32,
            snr: packet.snr,
            channel: packet.if_chain as u32,
            rf_chain: packet.rf_chain as u32,
            gw_time: match gps::cnt2time(packet.count_us) {
                Ok(v) => {
                    let v = v.duration_since(UNIX_EPOCH).unwrap();
                    Some(prost_types::Timestamp {
                        seconds: v.as_secs() as i64,
                        nanos: v.subsec_nanos() as i32,
                    })
                }
                Err(err) => {
                    debug!(
                        "Could not get GPS time, uplink_id: {}, error: {}",
                        uplink_id, err
                    );

                    if time_fallback {
                        Some(prost_types::Timestamp::from(SystemTime::now()))
                    } else {
                        None
                    }
                }
            },
            fine_time_since_gps_epoch: match packet.ftime_received {
                true => Some(prost_types::Duration {
                    nanos: packet.ftime as i32,
                    seconds: time_since_gps_epoch
                        .as_ref()
                        .map(|v| v.seconds)
                        .unwrap_or_default(),
                }),
                false => None,
            },
            time_since_gps_epoch,
            crc_status: match packet.status {
                hal::CRC::CRCOk => gw::CrcStatus::CrcOk,
                hal::CRC::BadCRC => gw::CrcStatus::BadCrc,
                hal::CRC::NoCRC | hal::CRC::Undefined => gw::CrcStatus::NoCrc,
            }
            .into(),
            ..Default::default()
        }),
        ..Default::default()
    })
}

pub fn downlink_from_proto(df: &gw::DownlinkFrameItem) -> Result<hal::TxPacket> {
    let mut data: [u8; 256] = [0; 256];
    let mut data_slice = df.phy_payload.clone();
    data_slice.resize(data.len(), 0);
    data.copy_from_slice(&data_slice);

    let tx_info = match df.tx_info.as_ref() {
        Some(v) => v,
        None => return Err(anyhow!("tx_info must not be blank")),
    };

    let mut packet = hal::TxPacket {
        freq_hz: tx_info.frequency,
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

    if let Some(timing) = &tx_info.timing {
        if let Some(params) = &timing.parameters {
            match params {
                gw::timing::Parameters::Immediately(_) => {
                    packet.tx_mode = hal::TxMode::Immediate;
                }
                gw::timing::Parameters::Delay(v) => {
                    packet.tx_mode = hal::TxMode::Timestamped;

                    let ctx = &tx_info.context;
                    if ctx.len() != 4 {
                        return Err(anyhow!("context must be exactly 4 bytes"));
                    }

                    match &v.delay {
                        Some(v) => {
                            let mut array = [0; 4];
                            array.copy_from_slice(ctx);
                            packet.count_us = u32::from_be_bytes(array).wrapping_add(
                                (Duration::from_secs(v.seconds as u64)
                                    + Duration::from_nanos(v.nanos as u64))
                                .as_micros() as u32,
                            );
                        }
                        None => {
                            return Err(anyhow!("delay must not be null"));
                        }
                    }
                }
                gw::timing::Parameters::GpsEpoch(v) => {
                    packet.tx_mode = hal::TxMode::Timestamped;

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
                            return Err(anyhow!("time_since_gps_epoch must not be null"));
                        }
                    }
                }
            }
        }
    }

    if let Some(modulation) = &tx_info.modulation {
        if let Some(params) = &modulation.parameters {
            match params {
                gw::modulation::Parameters::Lora(v) => {
                    packet.modulation = hal::Modulation::LoRa;
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
                        _ => return Err(anyhow!("unexpected spreading-factor")),
                    };
                    packet.coderate = match v.code_rate() {
                        gw::CodeRate::Cr45 => hal::CodeRate::LoRa4_5,
                        gw::CodeRate::Cr46 => hal::CodeRate::LoRa4_6,
                        gw::CodeRate::Cr47 => hal::CodeRate::LoRa4_7,
                        gw::CodeRate::Cr48 => hal::CodeRate::LoRa4_8,
                        _ => hal::CodeRate::Undefined,
                    };
                    packet.invert_pol = v.polarization_inversion;
                    packet.preamble = v.preamble as u16;
                    packet.no_crc = v.no_crc;
                }
                gw::modulation::Parameters::Fsk(v) => {
                    packet.modulation = hal::Modulation::FSK;
                    packet.datarate = hal::DataRate::FSK(v.datarate);
                    packet.f_dev = (v.frequency_deviation / 1000) as u8;
                }
                gw::modulation::Parameters::LrFhss(_) => {
                    return Err(anyhow!("LR-FHSS is not supported for downlink"));
                }
            }
        }
    }

    Ok(packet)
}

pub fn downlink_to_tx_info_proto(packet: &hal::TxPacket) -> Result<gw::DownlinkTxInfo> {
    Ok(gw::DownlinkTxInfo {
        frequency: packet.freq_hz,
        modulation: Some(gw::Modulation {
            parameters: match packet.modulation {
                hal::Modulation::LoRa => {
                    Some(gw::modulation::Parameters::Lora(gw::LoraModulationInfo {
                        bandwidth: packet.bandwidth,
                        spreading_factor: match packet.datarate {
                            hal::DataRate::SF5 => 5,
                            hal::DataRate::SF6 => 6,
                            hal::DataRate::SF7 => 7,
                            hal::DataRate::SF8 => 8,
                            hal::DataRate::SF9 => 9,
                            hal::DataRate::SF10 => 10,
                            hal::DataRate::SF11 => 11,
                            hal::DataRate::SF12 => 12,
                            _ => {
                                return Err(anyhow!("unexpected spreading-factor"));
                            }
                        },
                        code_rate: match packet.coderate {
                            hal::CodeRate::LoRa4_5 => gw::CodeRate::Cr45,
                            hal::CodeRate::LoRa4_6 => gw::CodeRate::Cr46,
                            hal::CodeRate::LoRa4_7 => gw::CodeRate::Cr47,
                            hal::CodeRate::LoRa4_8 => gw::CodeRate::Cr48,
                            hal::CodeRate::Undefined => gw::CodeRate::CrUndefined,
                        }
                        .into(),
                        ..Default::default()
                    }))
                }
                hal::Modulation::FSK => {
                    Some(gw::modulation::Parameters::Fsk(gw::FskModulationInfo {
                        datarate: match packet.datarate {
                            hal::DataRate::FSK(v) => v,
                            _ => return Err(anyhow!("unexpected datarate")),
                        },
                        ..Default::default()
                    }))
                }
                hal::Modulation::Undefined => None,
            },
        }),
        ..Default::default()
    })
}
