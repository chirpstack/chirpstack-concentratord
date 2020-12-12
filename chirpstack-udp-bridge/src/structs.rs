use std::convert::TryFrom;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const PROTOCOL_VERSION: u8 = 0x02;

pub enum CRC {
    NoCRC,
    OK,
    Fail,
}

impl Serialize for CRC {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CRC::NoCRC => serializer.serialize_i32(0),
            CRC::OK => serializer.serialize_i32(1),
            CRC::Fail => serializer.serialize_i32(-1),
        }
    }
}

pub enum Modulation {
    LoRa,
    Fsk,
}

impl Serialize for Modulation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Modulation::LoRa => serializer.serialize_str(&"LORA"),
            Modulation::Fsk => serializer.serialize_str(&"FSK"),
        }
    }
}

impl<'de> Deserialize<'de> for Modulation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "LORA" => Ok(Modulation::LoRa),
            "FSK" => Ok(Modulation::Fsk),
            _ => Err(D::Error::custom("unexpected value"))?,
        }
    }
}

pub enum DataRate {
    LoRa(u32, u32), // SF and BW (kHz)
    FSK(u32),       // bitrate
}

impl Serialize for DataRate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            DataRate::LoRa(sf, bw) => serializer.serialize_str(&format!("SF{}BW{}", sf, bw / 1000)),
            DataRate::FSK(bitrate) => serializer.serialize_u32(*bitrate),
        }
    }
}

impl<'de> Deserialize<'de> for DataRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let s: Vec<&str> = s.split(char::is_alphabetic).collect();
            if s.len() != 5 {
                return Err(D::Error::custom("invalid datarate string"));
            }

            let sf: u32 = match s[2].parse() {
                Ok(v) => v,
                Err(err) => {
                    return Err(D::Error::custom(format!("parse sf error: {}", err)));
                }
            };
            let bw: u32 = match s[4].parse() {
                Ok(v) => v,
                Err(err) => {
                    return Err(D::Error::custom(format!("parse bw error: {}", err)));
                }
            };

            return Ok(DataRate::LoRa(sf, bw * 1000));
        } else {
            let bitrate = u32::deserialize(deserializer)?;
            return Ok(DataRate::FSK(bitrate));
        }
    }
}

pub enum CodeRate {
    Undefined,
    LoRa4_5,
    LoRa4_6,
    LoRa4_7,
    LoRa4_8,
}

impl Serialize for CodeRate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CodeRate::LoRa4_5 => serializer.serialize_str(&"4/5"),
            CodeRate::LoRa4_6 => serializer.serialize_str(&"4/6"),
            CodeRate::LoRa4_7 => serializer.serialize_str(&"4/7"),
            CodeRate::LoRa4_8 => serializer.serialize_str(&"4/8"),
            _ => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for CodeRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "4/5" => Ok(CodeRate::LoRa4_5),
            "4/6" => Ok(CodeRate::LoRa4_6),
            "4/7" => Ok(CodeRate::LoRa4_7),
            "4/8" => Ok(CodeRate::LoRa4_8),
            _ => Ok(CodeRate::Undefined),
        }
    }
}

pub struct PushData {
    pub random_token: u16,
    pub gateway_id: [u8; 8],
    pub payload: PushDataPayload,
}

impl PushData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();

        b.push(PROTOCOL_VERSION);
        b.append(&mut self.random_token.to_be_bytes().to_vec());
        b.push(0x00);
        b.append(&mut self.gateway_id.to_vec());

        let mut j = serde_json::to_vec(&self.payload).unwrap();
        b.append(&mut j);

        return b;
    }
}

#[derive(Serialize)]
pub struct PushDataPayload {
    pub rxpk: Vec<RXPK>,
    pub stat: Option<Stat>,
}

#[derive(Serialize)]
pub struct RXPK {
    /// UTC time of pkt RX, us precision, ISO 8601 'compact' format
    #[serde(with = "compact_time_format")]
    pub time: DateTime<Utc>,
    /// GPS time of pkt RX, number of milliseconds since 06.Jan.1980
    pub tmms: Option<u64>,
    /// Internal timestamp of "RX finished" event (32b unsigned)
    pub tmst: u32,
    /// RX central frequency in MHz (unsigned float, Hz precision)
    pub freq: f64,
    /// Concentrator "IF" channel used for RX (unsigned integer)
    pub chan: u32,
    /// Concentrator "RF chain" used for RX (unsigned integer)
    pub rfch: u32,
    /// CRC status: 1 = OK, -1 = fail, 0 = no CRC
    pub stat: CRC,
    /// Modulation identifier "LORA" or "FSK"
    pub modu: Modulation,
    /// LoRa datarate identifier (eg. SF12BW500)}
    pub datr: DataRate,
    /// LoRa coding rate.
    pub codr: Option<CodeRate>,
    /// RSSI in dBm (signed integer, 1 dB precision).
    pub rssi: i32,
    /// Lora SNR ratio in dB (signed float, 0.1 dB precision).
    pub lsnr: Option<f32>,
    /// RF packet payload size in bytes (unsigned integer).
    pub size: u8,
    /// Base64 encoded RF packet payload, padded.
    pub data: String,
}

impl RXPK {
    pub fn from_proto(up: &chirpstack_api::gw::UplinkFrame) -> Result<Self, String> {
        let rx_info = match &up.rx_info {
            Some(v) => v,
            None => {
                return Err("rx_info must not be None".to_string());
            }
        };

        let tx_info = match &up.tx_info {
            Some(v) => v,
            None => {
                return Err("rx_info must not be None".to_string());
            }
        };

        Ok(RXPK {
            time: DateTime::from(match &rx_info.time {
                Some(v) => match SystemTime::try_from(v.clone()) {
                    Ok(vv) => vv,
                    Err(_) => SystemTime::now(),
                },
                None => SystemTime::now(),
            }),
            tmms: match &rx_info.time_since_gps_epoch {
                Some(v) => Some((v.seconds * 1000) as u64 + (v.nanos / 1000000) as u64),
                None => None,
            },
            tmst: {
                let mut bytes: [u8; 4] = [0; 4];
                bytes.copy_from_slice(&rx_info.context);
                u32::from_be_bytes(bytes)
            },
            freq: tx_info.frequency as f64 / 1000000.0,
            chan: rx_info.channel,
            rfch: rx_info.rf_chain,
            stat: match &rx_info.crc_status() {
                chirpstack_api::gw::CrcStatus::NoCrc => CRC::NoCRC,
                chirpstack_api::gw::CrcStatus::BadCrc => CRC::Fail,
                chirpstack_api::gw::CrcStatus::CrcOk => CRC::OK,
            },
            modu: match &tx_info.modulation_info {
                Some(v) => match v {
                    chirpstack_api::gw::uplink_tx_info::ModulationInfo::LoraModulationInfo(_) => {
                        Modulation::LoRa
                    }
                    chirpstack_api::gw::uplink_tx_info::ModulationInfo::FskModulationInfo(_) => {
                        Modulation::Fsk
                    }
                },
                None => {
                    return Err("modulation_info must not be None".to_string());
                }
            },
            datr: match &tx_info.modulation_info {
                Some(v) => match v {
                    chirpstack_api::gw::uplink_tx_info::ModulationInfo::LoraModulationInfo(vv) => {
                        DataRate::LoRa(vv.spreading_factor, vv.bandwidth)
                    }
                    chirpstack_api::gw::uplink_tx_info::ModulationInfo::FskModulationInfo(vv) => {
                        DataRate::FSK(vv.datarate)
                    }
                },
                None => {
                    return Err("modulation_info must not be None".to_string());
                }
            },
            codr: match &tx_info.modulation_info {
                Some(v) => match v {
                    chirpstack_api::gw::uplink_tx_info::ModulationInfo::LoraModulationInfo(vv) => {
                        match vv.code_rate.as_str() {
                            "4/5" => Some(CodeRate::LoRa4_5),
                            "4/6" => Some(CodeRate::LoRa4_6),
                            "4/7" => Some(CodeRate::LoRa4_7),
                            "4/8" => Some(CodeRate::LoRa4_8),
                            _ => None,
                        }
                    }
                    _ => None,
                },
                None => None,
            },
            rssi: rx_info.rssi,
            lsnr: match &tx_info.modulation_info {
                Some(v) => match v {
                    chirpstack_api::gw::uplink_tx_info::ModulationInfo::LoraModulationInfo(_) => {
                        Some(rx_info.lora_snr as f32)
                    }
                    _ => None,
                },
                None => None,
            },
            size: up.phy_payload.len() as u8,
            data: base64::encode(up.phy_payload.clone()),
        })
    }
}

#[derive(Serialize)]
pub struct Stat {
    /// UTC 'system' time of the gateway, ISO 8601 'expanded' format.
    #[serde(with = "expanded_time_format")]
    pub time: DateTime<Utc>,
    /// GPS latitude of the gateway in degree (float, N is +).
    pub lati: f64,
    /// GPS latitude of the gateway in degree (float, E is +).
    pub long: f64,
    /// GPS altitude of the gateway in meter RX (integer).
    pub alti: u32,
    /// Number of radio packets received (unsigned integer).
    pub rxnb: u32,
    /// Number of radio packets received with a valid PHY CRC.
    pub rxok: u32,
    /// Number of radio packets forwarded (unsigned integer).
    pub rxfw: u32,
    /// Percentage of upstream datagrams that were acknowledged.
    pub ackr: f32,
    /// Number of downlink datagrams received (unsigned integer).
    pub dwnb: u32,
    /// Number of packets emitted (unsigned integer).
    pub txnb: u32,
}

impl Stat {
    pub fn from_proto(stats: &chirpstack_api::gw::GatewayStats) -> Result<Self, String> {
        Ok(Stat {
            time: DateTime::from(match &stats.time {
                Some(v) => match SystemTime::try_from(v.clone()) {
                    Ok(vv) => vv,
                    Err(_) => SystemTime::now(),
                },
                None => SystemTime::now(),
            }),
            lati: match &stats.location {
                Some(v) => v.latitude,
                None => 0.0,
            },
            long: match &stats.location {
                Some(v) => v.longitude,
                None => 0.0,
            },
            alti: match &stats.location {
                Some(v) => v.altitude as u32,
                None => 0,
            },
            rxnb: stats.rx_packets_received,
            rxok: stats.rx_packets_received_ok,
            rxfw: 0,
            ackr: 0.0,
            dwnb: stats.tx_packets_received,
            txnb: stats.tx_packets_emitted,
        })
    }
}

pub struct PushAck {
    pub random_token: u16,
}

impl PushAck {
    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() != 4 {
            return Err(format!("expected 4 bytes, got: {}", b.len()).to_string());
        }

        if b[0] != PROTOCOL_VERSION {
            return Err(format!(
                "expected protocol version: {}, got: {}",
                PROTOCOL_VERSION, b[0]
            )
            .to_string());
        }

        if b[3] != 0x01 {
            return Err(format!("invalid identifier: {}", b[3]).to_string());
        }

        let mut rt: [u8; 2] = [0; 2];
        rt.copy_from_slice(&b[1..3]);

        Ok(PushAck {
            random_token: u16::from_be_bytes(rt),
        })
    }
}

pub struct PullData {
    pub random_token: u16,
    pub gateway_id: [u8; 8],
}

impl PullData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b: Vec<u8> = Vec::with_capacity(12);
        b.push(PROTOCOL_VERSION);
        b.append(&mut self.random_token.to_be_bytes().to_vec());
        b.push(0x02);
        b.append(&mut self.gateway_id.to_vec());

        return b;
    }
}

pub struct PullAck {
    pub random_token: u16,
}

impl PullAck {
    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() != 4 {
            return Err(format!("expected 4 bytes, got: {}", b.len()).to_string());
        }

        if b[0] != PROTOCOL_VERSION {
            return Err(format!(
                "expected protocol version: {}, got: {}",
                PROTOCOL_VERSION, b[0]
            )
            .to_string());
        }

        if b[3] != 0x04 {
            return Err(format!("invalid identifier: {}", b[3]).to_string());
        }

        let mut rt: [u8; 2] = [0; 2];
        rt.copy_from_slice(&b[1..3]);

        Ok(PullAck {
            random_token: u16::from_be_bytes(rt),
        })
    }
}

pub struct PullResp {
    pub random_token: u16,
    pub payload: PullRespPayload,
}

impl PullResp {
    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() < 5 {
            return Err(format!("expected at least 5 bytes, got: {}", b.len()).to_string());
        }

        if b[0] != PROTOCOL_VERSION {
            return Err(format!(
                "expected protocol version: {}, got: {}",
                PROTOCOL_VERSION, b[0]
            )
            .to_string());
        }

        if b[3] != 0x03 {
            return Err(format!("invalid identifier: {}", b[3]).to_string());
        }

        let mut rt: [u8; 2] = [0; 2];
        rt.copy_from_slice(&b[1..3]);

        let pl: PullRespPayload = match serde_json::from_slice(&b[4..]) {
            Ok(v) => v,
            Err(err) => {
                return Err(err.to_string());
            }
        };

        Ok(PullResp {
            random_token: u16::from_be_bytes(rt),
            payload: pl,
        })
    }
}

#[derive(Deserialize)]
pub struct PullRespPayload {
    pub txpk: TXPK,
}

#[derive(Deserialize)]
pub struct TXPK {
    /// Send packet immediately (will ignore tmst & time).
    pub imme: bool,
    /// Send packet on a certain timestamp value (will ignore time).
    pub tmst: Option<u32>,
    /// Send packet at a certain GPS time (GPS synchronization required).
    pub tmms: Option<u64>,
    /// TX central frequency in MHz (unsigned float, Hz precision).
    pub freq: f64,
    /// Concentrator "RF chain" used for TX (unsigned integer).
    pub rfch: u8,
    /// TX output power in dBm (unsigned integer, dBm precision).
    pub powe: u8,
    /// Modulation identifier "LORA" or "FSK".
    pub modu: Modulation,
    /// LoRa datarate identifier (eg. SF12BW500).
    pub datr: DataRate,
    /// LoRa ECC coding rate identifier.
    pub codr: CodeRate,
    /// FSK frequency deviation (unsigned integer, in Hz) .
    pub fdev: Option<u32>,
    /// Lora modulation polarization inversion.
    pub ipol: Option<bool>,
    /// RF preamble size (unsigned integer).
    pub prea: Option<u8>,
    /// RF packet payload size in bytes (unsigned integer).
    pub size: u8,
    /// Base64 encoded RF packet payload, padding optional.
    pub data: String,
    /// If true, disable the CRC of the physical layer (optional).
    pub ncrc: Option<bool>,
}

impl TXPK {
    pub fn to_proto(
        &self,
        downlink_id: Vec<u8>,
        gateway_id: Vec<u8>,
    ) -> Result<chirpstack_api::gw::DownlinkFrame, String> {
        // TXInfo
        let mut tx_info = chirpstack_api::gw::DownlinkTxInfo::default();
        tx_info.frequency = (self.freq * 1000000.0) as u32;
        tx_info.power = self.powe as i32;

        // TXInfo: set timing related data
        if self.imme {
            tx_info.set_timing(chirpstack_api::gw::DownlinkTiming::Immediately);
            tx_info.timing_info = Some(
                chirpstack_api::gw::downlink_tx_info::TimingInfo::ImmediatelyTimingInfo(
                    chirpstack_api::gw::ImmediatelyTimingInfo {},
                ),
            );
        } else if self.tmst.is_some() {
            tx_info.set_timing(chirpstack_api::gw::DownlinkTiming::Delay);
            tx_info.timing_info = Some(
                chirpstack_api::gw::downlink_tx_info::TimingInfo::DelayTimingInfo(
                    chirpstack_api::gw::DelayTimingInfo {
                        delay: Some(prost_types::Duration {
                            seconds: 0,
                            nanos: 0,
                        }),
                    },
                ),
            );
            tx_info.context = self.tmst.unwrap().to_be_bytes().to_vec();
        } else if self.tmms.is_some() {
            tx_info.set_timing(chirpstack_api::gw::DownlinkTiming::GpsEpoch);
            tx_info.timing_info = Some(
                chirpstack_api::gw::downlink_tx_info::TimingInfo::GpsEpochTimingInfo(
                    chirpstack_api::gw::GpsEpochTimingInfo {
                        time_since_gps_epoch: Some(prost_types::Duration::from(
                            Duration::from_millis(self.tmms.unwrap()),
                        )),
                    },
                ),
            );
        } else {
            return Err("no timing information found".to_string());
        }

        // TXInfo: set modulation related info
        match self.modu {
            Modulation::LoRa => {
                tx_info.set_modulation(chirpstack_api::common::Modulation::Lora);
                match self.datr {
                    DataRate::LoRa(sf, bw) => {
                        tx_info.modulation_info =
                    Some(chirpstack_api::gw::downlink_tx_info::ModulationInfo::LoraModulationInfo(
                        chirpstack_api::gw::LoRaModulationInfo {
                            bandwidth: bw,
                            spreading_factor: sf,
                            code_rate: match self.codr {
                                CodeRate::LoRa4_5 => "4/5".to_string(),
                                CodeRate::LoRa4_6 => "4/5".to_string(),
                                CodeRate::LoRa4_7 => "4/5".to_string(),
                                CodeRate::LoRa4_8 => "4/5".to_string(),
                                CodeRate::Undefined => "".to_string(),
                            },
                            polarization_inversion: match self.ipol {
                                Some(v) => v,
                                None => true,
                            },
                        },
                    ));
                    }
                    _ => {
                        return Err("LoRa DataRate expected".to_string());
                    }
                }
            }
            Modulation::Fsk => {
                tx_info.set_modulation(chirpstack_api::common::Modulation::Fsk);
                match self.datr {
                    DataRate::FSK(v) => {
                        tx_info.modulation_info = Some(
                            chirpstack_api::gw::downlink_tx_info::ModulationInfo::FskModulationInfo(
                                chirpstack_api::gw::FskModulationInfo {
                                    datarate: v,
                                    frequency_deviation: match self.fdev {
                                        Some(vv) => vv,
                                        None => {
                                            return Err("fdev must not be None".to_string());
                                        }
                                    },
                                },
                            ),
                        );
                    }
                    _ => {
                        return Err("FSK DataRate expected".to_string());
                    }
                }
            }
        }

        return Ok(chirpstack_api::gw::DownlinkFrame {
            downlink_id: downlink_id,
            gateway_id: gateway_id,
            items: vec![chirpstack_api::gw::DownlinkFrameItem {
                tx_info: Some(tx_info),
                phy_payload: match base64::decode(&self.data) {
                    Ok(v) => v,
                    Err(err) => {
                        return Err(err.to_string());
                    }
                },
            }],
            ..Default::default()
        });
    }
}

pub struct TxAck {
    pub random_token: u16,
    pub gateway_id: [u8; 8],
    pub payload: TxAckPayload,
}

impl TxAck {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();

        b.push(PROTOCOL_VERSION);
        b.append(&mut self.random_token.to_be_bytes().to_vec());
        b.push(0x05);
        b.append(&mut self.gateway_id.to_vec());

        let mut j = serde_json::to_vec(&self.payload).unwrap();
        b.append(&mut j);

        return b;
    }
}

#[derive(Serialize)]
pub struct TxAckPayload {
    pub txpk_ack: TxAckPayloadError,
}

#[derive(Serialize)]
pub struct TxAckPayloadError {
    pub error: String,
}

// see: https://serde.rs/custom-date-format.html
mod expanded_time_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S %Z";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }
}

mod compact_time_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    const FORMAT: &'static str = "%+";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }
}
