use std::time::{Duration, SystemTime};

use anyhow::Result;
use chirpstack_api::gw;
use libconcentratord::jitqueue;
use libloragw_2g4::hal;
use rand::Rng;

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
            hal::TxMode::CWOn => panic!("CWOn is not supported in queue"),
            hal::TxMode::CWOff => panic!("CWOff is not supported in queue"),
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

    fn set_count_us(&mut self, cout_us: u32) {
        self.0.count_us = cout_us;
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
    let mut rng = rand::thread_rng();

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
                            },
                            code_rate: match packet.coderate {
                                hal::CodeRate::LoRa4_5 => gw::CodeRate::Cr45,
                                hal::CodeRate::LoRa4_6 => gw::CodeRate::Cr46,
                                hal::CodeRate::LoRa4_7 => gw::CodeRate::Cr47,
                                hal::CodeRate::LoRa4_8 => gw::CodeRate::Cr48,
                                hal::CodeRate::LoRaLi4_5 => gw::CodeRate::CrLi45,
                                hal::CodeRate::LoRaLi4_6 => gw::CodeRate::CrLi46,
                                hal::CodeRate::LoRaLi4_8 => gw::CodeRate::CrLi48,
                            }
                            .into(),
                            ..Default::default()
                        }))
                    }
                },
            }),
        }),
        rx_info: Some(gw::UplinkRxInfo {
            uplink_id: rng.gen(),
            context: packet.count_us.to_be_bytes().to_vec(),
            gateway_id: hex::encode(gateway_id),
            rssi: packet.rssi as i32,
            snr: packet.snr,
            crc_status: match packet.status {
                hal::CRC::CRCOk => gw::CrcStatus::CrcOk,
                hal::CRC::BadCRC => gw::CrcStatus::BadCrc,
                hal::CRC::NoCRC | hal::CRC::Undefined => gw::CrcStatus::NoCrc,
            }
            .into(),
            gw_time: if time_fallback {
                Some(prost_types::Timestamp::from(SystemTime::now()))
            } else {
                None
            },
            ..Default::default()
        }),
        ..Default::default()
    })
}

pub fn downlink_from_proto(
    lorawan_public: bool,
    df: &gw::DownlinkFrameItem,
) -> Result<hal::TxPacket> {
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
        rf_power: tx_info.power as i8,
        preamble: 0,
        sync_word: match lorawan_public {
            true => 0x21,
            false => 0x12,
        },
        no_crc: false,
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
                gw::timing::Parameters::GpsEpoch(_) => {
                    return Err(anyhow!("gps epoch timing is not implemented"));
                }
            }
        }
    }

    if let Some(modulation) = &tx_info.modulation {
        if let Some(params) = &modulation.parameters {
            match params {
                gw::modulation::Parameters::Lora(v) => {
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
                        gw::CodeRate::CrLi45 => hal::CodeRate::LoRaLi4_5,
                        gw::CodeRate::CrLi46 => hal::CodeRate::LoRaLi4_6,
                        gw::CodeRate::CrLi48 => hal::CodeRate::LoRaLi4_8,
                        _ => return Err(anyhow!("unexpected coderate")),
                    };
                    packet.preamble = if v.preamble > 0 {
                        v.preamble as u16
                    } else {
                        match v.spreading_factor {
                            5 => 12,
                            6 => 12,
                            7 => 8,
                            8 => 8,
                            9 => 8,
                            10 => 8,
                            11 => 8,
                            12 => 8,
                            _ => return Err(anyhow!("unexpected spreading-factor")),
                        }
                    };
                    packet.no_crc = v.no_crc;
                    packet.invert_pol = v.polarization_inversion;
                }
                _ => {
                    return Err(anyhow!("only LORA modulation is implemented"));
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
            parameters: Some(gw::modulation::Parameters::Lora(gw::LoraModulationInfo {
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
                },
                code_rate: match packet.coderate {
                    hal::CodeRate::LoRa4_5 => gw::CodeRate::Cr45,
                    hal::CodeRate::LoRa4_6 => gw::CodeRate::Cr46,
                    hal::CodeRate::LoRa4_7 => gw::CodeRate::Cr47,
                    hal::CodeRate::LoRa4_8 => gw::CodeRate::Cr48,
                    hal::CodeRate::LoRaLi4_5 => gw::CodeRate::CrLi45,
                    hal::CodeRate::LoRaLi4_6 => gw::CodeRate::CrLi46,
                    hal::CodeRate::LoRaLi4_8 => gw::CodeRate::CrLi48,
                }
                .into(),
                ..Default::default()
            })),
        }),
        ..Default::default()
    })
}
