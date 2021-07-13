use std::convert::TryInto;
use std::ffi::CStr;
use std::time::Duration;

use super::{mutex, wrapper};

// ConvertBandwidth is a trait to convert the bandwidth from / to the HAL
// enum constants. A trait is needed as it is the only way to add methods to
// a type alias.
trait ConvertBandwidth {
    fn from_hal(_: u8) -> u32;
    fn to_hal(&self) -> u8;
}

/// Bandwidth in Hz.
type Bandwidth = u32;

impl ConvertBandwidth for Bandwidth {
    fn from_hal(bandwidth: u8) -> u32 {
        match bandwidth as u32 {
            wrapper::BW_500KHZ => 500000,
            wrapper::BW_250KHZ => 250000,
            wrapper::BW_125KHZ => 125000,
            wrapper::BW_62K5HZ => 62500,
            wrapper::BW_31K2HZ => 31200,
            wrapper::BW_7K8HZ => 7800,
            _ => 0,
        }
    }

    fn to_hal(&self) -> u8 {
        return match self {
            500000 => wrapper::BW_500KHZ,
            250000 => wrapper::BW_250KHZ,
            125000 => wrapper::BW_125KHZ,
            62500 => wrapper::BW_62K5HZ,
            31200 => wrapper::BW_31K2HZ,
            15600 => wrapper::BW_15K6HZ,
            7800 => wrapper::BW_7K8HZ,
            _ => wrapper::BW_UNDEFINED,
        } as u8;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RadioType {
    SX1255,
    SX1257,
    SX1272,
    SX1276,
}

impl RadioType {
    fn to_hal(&self) -> wrapper::lgw_radio_type_e {
        match self {
            RadioType::SX1255 => wrapper::lgw_radio_type_e_LGW_RADIO_TYPE_SX1255,
            RadioType::SX1257 => wrapper::lgw_radio_type_e_LGW_RADIO_TYPE_SX1257,
            RadioType::SX1272 => wrapper::lgw_radio_type_e_LGW_RADIO_TYPE_SX1272,
            RadioType::SX1276 => wrapper::lgw_radio_type_e_LGW_RADIO_TYPE_SX1276,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CRC {
    Undefined,
    NoCRC,
    BadCRC,
    CRCOk,
}

impl CRC {
    fn from_hal(status: u8) -> Self {
        match status as u32 {
            wrapper::STAT_NO_CRC => CRC::NoCRC,
            wrapper::STAT_CRC_BAD => CRC::BadCRC,
            wrapper::STAT_CRC_OK => CRC::CRCOk,
            _ => CRC::Undefined,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Modulation {
    Undefined,
    LoRa,
    FSK,
}

impl Modulation {
    fn from_hal(modulation: u8) -> Self {
        match modulation as u32 {
            wrapper::MOD_LORA => Modulation::LoRa,
            wrapper::MOD_FSK => Modulation::FSK,
            _ => Modulation::Undefined,
        }
    }
}

impl Modulation {
    fn to_hal(&self) -> u8 {
        return match self {
            Modulation::Undefined => wrapper::MOD_UNDEFINED,
            Modulation::LoRa => wrapper::MOD_LORA,
            Modulation::FSK => wrapper::MOD_FSK,
        } as u8;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DataRate {
    Undefined,
    SF7,
    SF8,
    SF9,
    SF10,
    SF11,
    SF12,
    MultiSF,
    FSK(u32),
}

impl DataRate {
    fn to_hal(&self) -> u32 {
        return match self {
            DataRate::Undefined => wrapper::DR_UNDEFINED,
            DataRate::SF7 => wrapper::DR_LORA_SF7,
            DataRate::SF8 => wrapper::DR_LORA_SF8,
            DataRate::SF9 => wrapper::DR_LORA_SF9,
            DataRate::SF10 => wrapper::DR_LORA_SF10,
            DataRate::SF11 => wrapper::DR_LORA_SF11,
            DataRate::SF12 => wrapper::DR_LORA_SF12,
            DataRate::MultiSF => wrapper::DR_LORA_MULTI,
            DataRate::FSK(dr) => *dr,
        } as u32;
    }

    fn from_hal(datarate: u32) -> Self {
        match datarate {
            wrapper::DR_UNDEFINED => DataRate::Undefined,
            wrapper::DR_LORA_SF7 => DataRate::SF7,
            wrapper::DR_LORA_SF8 => DataRate::SF8,
            wrapper::DR_LORA_SF9 => DataRate::SF9,
            wrapper::DR_LORA_SF10 => DataRate::SF10,
            wrapper::DR_LORA_SF11 => DataRate::SF11,
            wrapper::DR_LORA_SF12 => DataRate::SF12,
            _ => DataRate::FSK(datarate),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CodeRate {
    Undefined,
    LoRa4_5,
    LoRa4_6,
    LoRa4_7,
    LoRa4_8,
}

impl CodeRate {
    fn to_hal(&self) -> u8 {
        return match self {
            CodeRate::Undefined => wrapper::CR_UNDEFINED,
            CodeRate::LoRa4_5 => wrapper::CR_LORA_4_5,
            CodeRate::LoRa4_6 => wrapper::CR_LORA_4_6,
            CodeRate::LoRa4_7 => wrapper::CR_LORA_4_7,
            CodeRate::LoRa4_8 => wrapper::CR_LORA_4_8,
        } as u8;
    }

    fn from_hal(coderate: u8) -> Self {
        match coderate as u32 {
            wrapper::CR_LORA_4_5 => CodeRate::LoRa4_5,
            wrapper::CR_LORA_4_6 => CodeRate::LoRa4_6,
            wrapper::CR_LORA_4_7 => CodeRate::LoRa4_7,
            wrapper::CR_LORA_4_8 => CodeRate::LoRa4_8,
            _ => CodeRate::Undefined,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TxMode {
    Immediate,
    Timestamped,
    OnGPS,
}

impl TxMode {
    fn to_hal(&self) -> u8 {
        return match self {
            TxMode::Immediate => wrapper::IMMEDIATE,
            TxMode::Timestamped => wrapper::TIMESTAMPED,
            TxMode::OnGPS => wrapper::ON_GPS,
        } as u8;
    }
}

#[derive(PartialEq, Eq)]
pub enum StatusSelect {
    Tx,
    Rx,
}

impl StatusSelect {
    fn to_hal(&self) -> u8 {
        return match self {
            StatusSelect::Tx => wrapper::TX_STATUS,
            StatusSelect::Rx => wrapper::RX_STATUS,
        } as u8;
    }
}

pub enum StatusReturn {
    Tx(TxStatus),
    Rx(RxStatus),
}

#[derive(Debug)]
pub enum TxStatus {
    Unknown,
    Off,
    Free,
    Scheduled,
    Emitting,
}

impl TxStatus {
    fn from_hal(code: u8) -> Self {
        match code as u32 {
            wrapper::TX_OFF => TxStatus::Off,
            wrapper::TX_FREE => TxStatus::Free,
            wrapper::TX_SCHEDULED => TxStatus::Scheduled,
            wrapper::TX_EMITTING => TxStatus::Emitting,
            _ => TxStatus::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum RxStatus {
    Unknown,
    Off,
    On,
    Suspended,
}

impl RxStatus {
    fn from_hal(code: u8) -> Self {
        match code as u32 {
            wrapper::RX_OFF => RxStatus::Off,
            wrapper::RX_ON => RxStatus::On,
            wrapper::RX_SUSPENDED => RxStatus::Suspended,
            _ => RxStatus::Unknown,
        }
    }
}

/// Configuration structure for board specificities.
pub struct BoardConfig {
    /// Enable ONLY for *public* networks using the LoRa MAC protocol.
    pub lorawan_public: bool,
    /// Index of RF chain which provides clock to concentrator.
    pub clock_source: u8,
}

/// Configuration structure for a RF chain.
pub struct RxRfConfig {
    /// Enable or disable that RF chain.
    pub enable: bool,
    /// Center frequency of the radio in Hz.
    pub freq_hz: u32,
    /// Board-specific RSSI correction factor.
    pub rssi_offset: f32,
    /// Radio type for that RF chain (SX1255, SX1257....).
    pub radio_type: RadioType,
    /// Enable or disable TX on that RF chain.
    pub tx_enable: bool,
    /// TX notch filter frequency [126KHz..250KHz].
    pub tx_notch_freq: u32,
}

impl RxRfConfig {
    fn to_hal(&self) -> wrapper::lgw_conf_rxrf_s {
        wrapper::lgw_conf_rxrf_s {
            enable: self.enable,
            freq_hz: self.freq_hz,
            rssi_offset: self.rssi_offset,
            tx_enable: self.tx_enable,
            tx_notch_freq: self.tx_notch_freq,
            type_: self.radio_type.to_hal(),
        }
    }
}

/// Configuration structure for an IF chain.
pub struct RxIfConfig {
    /// Enable or disable that IF chain.
    pub enable: bool,
    /// To which RF chain is that IF chain associated.
    pub rf_chain: u8,
    /// Center frequ of the IF chain, relative to RF chain frequency.
    pub freq_hz: i32,
    /// RX bandwidth, 0 for default.
    pub bandwidth: Bandwidth,
    /// RX datarate, 0 for default.
    pub datarate: DataRate,
    /// Size of FSK sync word (number of bytes, 0 for default).
    pub sync_word_size: u8,
    /// FSK sync word (ALIGN RIGHT, eg. 0xC194C1).
    pub sync_word: u64,
}

impl RxIfConfig {
    fn to_hal(&self) -> wrapper::lgw_conf_rxif_s {
        wrapper::lgw_conf_rxif_s {
            enable: self.enable,
            rf_chain: self.rf_chain,
            freq_hz: self.freq_hz,
            bandwidth: self.bandwidth.to_hal(),
            datarate: self.datarate.to_hal(),
            sync_word_size: self.sync_word_size,
            sync_word: self.sync_word,
            ..Default::default()
        }
    }
}

impl Default for RxIfConfig {
    fn default() -> Self {
        RxIfConfig {
            enable: false,
            rf_chain: 0,
            freq_hz: 0,
            bandwidth: 0,
            datarate: DataRate::Undefined,
            sync_word_size: 0,
            sync_word: 0,
        }
    }
}

/// Structure containing all gains of Tx chain.
#[derive(Clone)]
pub struct TxGainConfig {
    /// 2 bits, control of the digital gain of SX1301.
    pub dig_gain: u8,
    /// 2 bits, control of the external PA (SX1301 I/O).
    pub pa_gain: u8,
    /// 2 bits, control of the radio DAC.
    pub dac_gain: u8,
    /// 4 bits, control of the radio mixer.
    pub mix_gain: u8,
    /// Measured TX power at the board connector, in dBm.
    pub rf_power: i8,
}

impl TxGainConfig {
    fn to_hal(&self) -> wrapper::lgw_tx_gain_s {
        wrapper::lgw_tx_gain_s {
            dig_gain: self.dig_gain,
            pa_gain: self.pa_gain,
            dac_gain: self.dac_gain,
            mix_gain: self.mix_gain,
            rf_power: self.rf_power,
        }
    }
}

/// Structure containing the metadata of a packet that was received and a pointer to the payload.
pub struct RxPacket {
    /// Central frequency of the IF chain.
    pub freq_hz: u32,
    /// By which IF chain was packet received.
    pub if_chain: u8,
    /// Status of the received packet.
    pub status: CRC,
    /// Internal concentrator counter for timestamping, 1 microsecond resolution.
    pub count_us: u32,
    /// Through which RF chain the packet was received.
    pub rf_chain: u8,
    /// Modulation used by the packet.
    pub modulation: Modulation,
    /// Modulation bandwidth (LoRa only).
    pub bandwidth: Bandwidth,
    /// RX datarate of the packet (SF for LoRa).
    pub datarate: DataRate,
    /// Error-correcting code of the packet (LoRa only).
    pub coderate: CodeRate,
    /// Average packet RSSI in dB.
    pub rssi: f32,
    /// Average packet SNR, in dB (LoRa only).
    pub snr: f32,
    /// Minimum packet SNR, in dB (LoRa only).
    pub snr_min: f32,
    /// Maximum packet SNR, in dB (LoRa only).
    pub snr_max: f32,
    /// CRC that was received in the payload.
    pub crc: u16,
    /// Payload size in bytes.
    pub size: u16,
    /// Buffer containing the payload.
    pub payload: [u8; 256],
}

impl RxPacket {
    fn from_hal(pkt: wrapper::lgw_pkt_rx_s) -> Self {
        RxPacket {
            freq_hz: pkt.freq_hz,
            if_chain: pkt.if_chain,
            status: CRC::from_hal(pkt.status),
            count_us: pkt.count_us,
            rf_chain: pkt.rf_chain,
            modulation: Modulation::from_hal(pkt.modulation),
            bandwidth: Bandwidth::from_hal(pkt.bandwidth),
            datarate: DataRate::from_hal(pkt.datarate),
            coderate: CodeRate::from_hal(pkt.coderate),
            rssi: pkt.rssi,
            snr: pkt.snr,
            snr_min: pkt.snr_min,
            snr_max: pkt.snr_max,
            crc: pkt.crc,
            size: pkt.size,
            payload: pkt.payload,
        }
    }
}

/// Structure containing the configuration of a packet to send and a pointer to the payload.
#[derive(Copy, Clone)]
pub struct TxPacket {
    /// Center frequency of TX.
    pub freq_hz: u32,
    /// Select on what event/time the TX is triggered.
    pub tx_mode: TxMode,
    /// Timestamp or delay in microseconds for TX trigger.
    pub count_us: u32,
    /// Through which RF chain will the packet be sent.
    pub rf_chain: u8,
    /// TX power, in dBm.
    pub rf_power: i8,
    /// Modulation to use for the packet.
    pub modulation: Modulation,
    /// Modulation bandwidth (LoRa only).
    pub bandwidth: Bandwidth,
    /// TX datarate (baudrate for FSK, SF for LoRa).
    pub datarate: DataRate,
    /// Error-correcting code of the packet (LoRa only).
    pub coderate: CodeRate,
    /// Invert signal polarity, for orthogonal downlinks (LoRa only).
    pub invert_pol: bool,
    /// Frequency deviation, in kHz (FSK only).
    pub f_dev: u8,
    /// Set the preamble length, 0 for default.
    pub preamble: u16,
    /// If true, do not send a CRC in the packet.
    pub no_crc: bool,
    /// If true, enable implicit header mode (LoRa), fixed length (FSK).
    pub no_header: bool,
    /// Payload size in bytes.
    pub size: u16,
    /// Buffer containing the payload.
    pub payload: [u8; 256],
}

impl Default for TxPacket {
    fn default() -> Self {
        TxPacket {
            freq_hz: 0,
            tx_mode: TxMode::Immediate,
            count_us: 0,
            rf_chain: 0,
            rf_power: 0,
            modulation: Modulation::Undefined,
            bandwidth: 0,
            datarate: DataRate::Undefined,
            coderate: CodeRate::Undefined,
            invert_pol: false,
            f_dev: 0,
            preamble: 0,
            no_crc: false,
            no_header: false,
            size: 0,
            payload: [0; 256],
        }
    }
}

impl TxPacket {
    fn to_hal(&self) -> wrapper::lgw_pkt_tx_s {
        wrapper::lgw_pkt_tx_s {
            freq_hz: self.freq_hz,
            tx_mode: self.tx_mode.to_hal(),
            count_us: self.count_us,
            rf_chain: self.rf_chain,
            rf_power: self.rf_power,
            modulation: self.modulation.to_hal(),
            bandwidth: self.bandwidth.to_hal(),
            datarate: self.datarate.to_hal(),
            coderate: self.coderate.to_hal(),
            invert_pol: self.invert_pol,
            f_dev: self.f_dev,
            preamble: self.preamble,
            no_crc: self.no_crc,
            no_header: self.no_header,
            size: self.size,
            payload: self.payload,
        }
    }
}

const MAX_PKT: usize = 8;

/// Configure the gateway board.
pub fn board_setconf(conf: &BoardConfig) -> Result<(), String> {
    let conf = wrapper::lgw_conf_board_s {
        lorawan_public: conf.lorawan_public,
        clksrc: conf.clock_source,
    };

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_board_setconf(conf) };
    if ret != 0 {
        return Err("lgw_board_setconf faild".to_string());
    }

    return Ok(());
}

/// Configure an RF chain (must configure before start).
/// rf_chain: number of the RF chain to configure (index starting at 0).
pub fn rxrf_setconf(rf_chain: u8, conf: &RxRfConfig) -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let conf = conf.to_hal();
    let ret = unsafe { wrapper::lgw_rxrf_setconf(rf_chain, conf) };
    if ret != 0 {
        return Err("lgw_rxrf_setconf failed".to_string());
    }

    return Ok(());
}

/// Configure an IF chain + modem (must configure before start).
/// rf_chain: number of the IF chain to configure (index starting at 0).
pub fn rxif_setconf(if_chain: u8, conf: &RxIfConfig) -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let conf = conf.to_hal();
    let ret = unsafe { wrapper::lgw_rxif_setconf(if_chain, conf) };
    if ret != 0 {
        return Err("lgw_rxif_setconf failed".to_string());
    }

    return Ok(());
}

/// Configure the Tx gain LUT.
pub fn txgain_setconf(txgain: &[TxGainConfig]) -> Result<(), String> {
    let mut conf = wrapper::lgw_tx_gain_lut_s {
        lut: [wrapper::lgw_tx_gain_s {
            ..Default::default()
        }; 16],
        size: txgain.len().try_into().unwrap(),
    };

    for (i, gain) in txgain.iter().enumerate() {
        conf.lut[i] = gain.to_hal();
    }

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_txgain_setconf(&mut conf) };
    if ret != 0 {
        return Err("lgw_txgain_setconf failed".to_string());
    }

    return Ok(());
}

/// Connect to the LoRa concentrator, reset it and configure it according to previously set
/// parameters.
pub fn start() -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_start() };
    if ret != 0 {
        return Err("lgw_start failed".to_string());
    }
    return Ok(());
}

/// Stop the LoRa concentrator and disconnect it.
pub fn stop() -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_stop() };
    if ret != 0 {
        return Err("lgw_stop failed".to_string());
    }
    return Ok(());
}

/// A non-blocking function that will fetch up to MAX_PKT (8) packets from the LoRa concentrator FIFO
/// and data buffer.
pub fn receive() -> Result<Vec<RxPacket>, String> {
    let mut packets: [wrapper::lgw_pkt_rx_s; MAX_PKT] = [Default::default(); MAX_PKT];

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_receive(MAX_PKT.try_into().unwrap(), packets.as_mut_ptr()) };
    if ret == -1 {
        return Err("lgw_receive failed".to_string());
    }

    let mut v: Vec<RxPacket> = Vec::new();

    for x in 0..ret {
        let pkt = packets[x as usize];

        v.push(RxPacket::from_hal(pkt));
    }

    return Ok(v);
}

/// Schedule a packet to be send immediately or after a delay depending on tx_mode.
pub fn send(pkt: &TxPacket) -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let pkt = pkt.to_hal();

    let ret = unsafe { wrapper::lgw_send(pkt) };
    if ret != 0 {
        return Err("lgw_send failed".to_string());
    }

    return Ok(());
}

/// Give the the status of different part of the LoRa concentrator.
pub fn status(select: StatusSelect) -> Result<StatusReturn, String> {
    let mut code = 0;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_status(select.to_hal(), &mut code) };
    if ret != 0 {
        return Err("lgw_status failed".to_string());
    }

    if select == StatusSelect::Tx {
        return Ok(StatusReturn::Tx(TxStatus::from_hal(code)));
    } else {
        return Ok(StatusReturn::Rx(RxStatus::from_hal(code)));
    }
}

/// Abort a currently scheduled or ongoing TX.
pub fn abort_tx() -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_abort_tx() };
    if ret != 0 {
        return Err("lgw_abort_tx failed".to_string());
    }
    return Ok(());
}

/// Return value of internal counter when latest event (eg GPS pulse) was captured.
pub fn get_trigcnt() -> Result<u32, String> {
    let mut cnt = 0;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_get_trigcnt(&mut cnt) };
    if ret != 0 {
        return Err("lgw_get_trigcnt failed".to_string());
    }

    return Ok(cnt);
}

/// Return time on air of given packet.
pub fn time_on_air(pkt: &TxPacket) -> Result<Duration, String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let mut pkt = pkt.to_hal();

    let ms = unsafe { wrapper::lgw_time_on_air(&mut pkt) };
    return Ok(Duration::from_millis(ms as u64));
}

/// Allow user to check the version/options of the library once compiled.
pub fn version_info() -> String {
    unsafe {
        CStr::from_ptr(wrapper::lgw_version_info())
            .to_string_lossy()
            .into_owned()
    }
}
