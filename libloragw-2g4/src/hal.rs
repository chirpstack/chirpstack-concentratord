use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::os::raw::c_char;
use std::time::Duration;

use super::{mutex, wrapper};

// ConvertBandwidth is a trait to convert the bandwidth from / to the HAL
// enum constants. A trait is needed as it is the only way to add methods to
// a type alias.
trait ConvertBandwidth {
    fn from_hal(_: u32) -> u32;
    fn to_hal(&self) -> u32;
}

/// Bandwidth in Hz.
type Bandwidth = u32;

// Please see:
// https://github.com/Lora-net/gateway_2g4_hal/issues/6
impl ConvertBandwidth for Bandwidth {
    fn from_hal(bandwidth: u32) -> u32 {
        match bandwidth {
            wrapper::e_bandwidth_BW_200KHZ => 203000,
            wrapper::e_bandwidth_BW_400KHZ => 406000,
            wrapper::e_bandwidth_BW_800KHZ => 812000,
            wrapper::e_bandwidth_BW_1600KHZ => 1625000,
            _ => 0,
        }
    }

    fn to_hal(&self) -> u32 {
        match self {
            203000 => wrapper::e_bandwidth_BW_200KHZ,
            406000 => wrapper::e_bandwidth_BW_400KHZ,
            812000 => wrapper::e_bandwidth_BW_800KHZ,
            1625000 => wrapper::e_bandwidth_BW_1600KHZ,
            _ => 0,
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
            wrapper::e_crc_status_STAT_UNDEFINED => CRC::Undefined,
            wrapper::e_crc_status_STAT_NO_CRC => CRC::NoCRC,
            wrapper::e_crc_status_STAT_CRC_BAD => CRC::BadCRC,
            wrapper::e_crc_status_STAT_CRC_OK => CRC::CRCOk,
            _ => CRC::Undefined,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Modulation {
    LoRa,
}

impl Modulation {
    fn from_hal(modulation: u32) -> Self {
        match modulation {
            wrapper::e_modulation_MOD_LORA => Modulation::LoRa,
            _ => Modulation::LoRa,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DataRate {
    SF5,
    SF6,
    SF7,
    SF8,
    SF9,
    SF10,
    SF11,
    SF12,
}

impl DataRate {
    fn to_hal(&self) -> u32 {
        match self {
            DataRate::SF5 => wrapper::e_spreading_factor_DR_LORA_SF5,
            DataRate::SF6 => wrapper::e_spreading_factor_DR_LORA_SF6,
            DataRate::SF7 => wrapper::e_spreading_factor_DR_LORA_SF7,
            DataRate::SF8 => wrapper::e_spreading_factor_DR_LORA_SF8,
            DataRate::SF9 => wrapper::e_spreading_factor_DR_LORA_SF9,
            DataRate::SF10 => wrapper::e_spreading_factor_DR_LORA_SF10,
            DataRate::SF11 => wrapper::e_spreading_factor_DR_LORA_SF11,
            DataRate::SF12 => wrapper::e_spreading_factor_DR_LORA_SF12,
        }
    }

    fn from_hal(datarate: u32) -> Self {
        match datarate {
            wrapper::e_spreading_factor_DR_LORA_SF5 => DataRate::SF5,
            wrapper::e_spreading_factor_DR_LORA_SF6 => DataRate::SF6,
            wrapper::e_spreading_factor_DR_LORA_SF7 => DataRate::SF7,
            wrapper::e_spreading_factor_DR_LORA_SF8 => DataRate::SF8,
            wrapper::e_spreading_factor_DR_LORA_SF9 => DataRate::SF9,
            wrapper::e_spreading_factor_DR_LORA_SF10 => DataRate::SF10,
            wrapper::e_spreading_factor_DR_LORA_SF11 => DataRate::SF11,
            wrapper::e_spreading_factor_DR_LORA_SF12 => DataRate::SF12,
            _ => DataRate::SF5,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CodeRate {
    LoRa4_5,
    LoRa4_6,
    LoRa4_7,
    LoRa4_8,
    LoRaLi4_5,
    LoRaLi4_6,
    LoRaLi4_8,
}

impl CodeRate {
    fn to_hal(&self) -> u32 {
        match self {
            CodeRate::LoRa4_5 => wrapper::e_coding_rate_CR_LORA_4_5,
            CodeRate::LoRa4_6 => wrapper::e_coding_rate_CR_LORA_4_6,
            CodeRate::LoRa4_7 => wrapper::e_coding_rate_CR_LORA_4_7,
            CodeRate::LoRa4_8 => wrapper::e_coding_rate_CR_LORA_4_8,
            CodeRate::LoRaLi4_5 => wrapper::e_coding_rate_CR_LORA_LI_4_5,
            CodeRate::LoRaLi4_6 => wrapper::e_coding_rate_CR_LORA_LI_4_6,
            CodeRate::LoRaLi4_8 => wrapper::e_coding_rate_CR_LORA_LI_4_8,
        }
    }

    fn from_hal(coderate: u32) -> Self {
        match coderate {
            wrapper::e_coding_rate_CR_LORA_4_5 => CodeRate::LoRa4_5,
            wrapper::e_coding_rate_CR_LORA_4_6 => CodeRate::LoRa4_6,
            wrapper::e_coding_rate_CR_LORA_4_7 => CodeRate::LoRa4_7,
            wrapper::e_coding_rate_CR_LORA_4_8 => CodeRate::LoRa4_8,
            wrapper::e_coding_rate_CR_LORA_LI_4_5 => CodeRate::LoRaLi4_5,
            wrapper::e_coding_rate_CR_LORA_LI_4_6 => CodeRate::LoRaLi4_6,
            wrapper::e_coding_rate_CR_LORA_LI_4_8 => CodeRate::LoRaLi4_8,
            _ => CodeRate::LoRa4_5,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TxMode {
    Timestamped,
    Immediate,
    OnGPS,
    CWOn,
    CWOff,
}

impl TxMode {
    fn to_hal(&self) -> u32 {
        match self {
            TxMode::Timestamped => wrapper::e_tx_mode_TIMESTAMPED,
            TxMode::Immediate => wrapper::e_tx_mode_IMMEDIATE,
            TxMode::OnGPS => wrapper::e_tx_mode_ON_GPS,
            TxMode::CWOn => wrapper::e_tx_mode_CW_ON,
            TxMode::CWOff => wrapper::e_tx_mode_CW_OFF,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum StatusSelect {
    Tx,
    Rx,
}

impl StatusSelect {
    fn to_hal(&self) -> u32 {
        match self {
            StatusSelect::Tx => wrapper::e_status_type_TX_STATUS,
            StatusSelect::Rx => wrapper::e_status_type_RX_STATUS,
        }
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
    fn from_hal(code: u32) -> Self {
        match code {
            wrapper::e_status_TX_STATUS_UNKNOWN => TxStatus::Unknown,
            wrapper::e_status_TX_OFF => TxStatus::Off,
            wrapper::e_status_TX_FREE => TxStatus::Free,
            wrapper::e_status_TX_SCHEDULED => TxStatus::Scheduled,
            wrapper::e_status_TX_EMITTING => TxStatus::Emitting,
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
    fn from_hal(code: u32) -> Self {
        match code {
            wrapper::e_status_RX_STATUS_UNKNOWN => RxStatus::Unknown,
            wrapper::e_status_RX_OFF => RxStatus::Off,
            wrapper::e_status_RX_ON => RxStatus::On,
            wrapper::e_status_RX_SUSPENDED => RxStatus::Suspended,
            _ => RxStatus::Unknown,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TemperatureSource {
    /// The temperature has been measured with an external sensor.
    Ext,
    /// The temperature has been measured by the gateway MCU.
    Mcu,
}

impl TemperatureSource {
    fn to_hal(&self) -> u32 {
        match self {
            TemperatureSource::Ext => wrapper::e_temperature_src_TEMP_SRC_EXT,
            TemperatureSource::Mcu => wrapper::e_temperature_src_TEMP_SRC_MCU,
        }
    }
}

/// Configuration structure for board specificities.
pub struct BoardConfig {
    /// Path to access the TTY device to connect to concentrator board.
    pub tty_path: String,
}

impl BoardConfig {
    fn to_hal(&self) -> Result<wrapper::lgw_conf_board_s, String> {
        let tty_path = CString::new(self.tty_path.clone()).unwrap();
        let tty_path = tty_path.as_bytes_with_nul();
        if tty_path.len() > 64 {
            return Err("tty_path max length is 64".to_string());
        }
        let mut tty_path_chars = [0; 64];
        for (i, b) in tty_path.iter().enumerate() {
            tty_path_chars[i] = *b as c_char;
        }

        return Ok(wrapper::lgw_conf_board_s {
            tty_path: tty_path_chars,
        });
    }
}

/// Configuration structure for a channel.
pub struct ChannelRxConfig {
    /// Enable or disable that channel.
    pub enable: bool,
    /// channel frequency in Hz.
    pub freq_hz: u32,
    /// RX bandwidth.
    pub bandwidth: Bandwidth,
    /// RX datarate.
    pub datarate: DataRate,
    /// RSSI offset to be applied on this channel.'
    pub rssi_offset: f32,
    /// Public network:0x21, Private network:0x12.
    pub sync_word: u8,
}

impl ChannelRxConfig {
    fn to_hal(&self) -> Result<wrapper::lgw_conf_channel_rx_s, String> {
        return Ok(wrapper::lgw_conf_channel_rx_s {
            enable: self.enable,
            freq_hz: self.freq_hz,
            bandwidth: self.bandwidth.to_hal(),
            datarate: self.datarate.to_hal(),
            rssi_offset: self.rssi_offset,
            sync_word: self.sync_word,
        });
    }
}

/// Configuration structure for TX.
pub struct ChannelTxConfig {
    /// Enable or disable that channel.
    pub enable: bool,
}

impl ChannelTxConfig {
    fn to_hal(&self) -> Result<wrapper::lgw_conf_channel_tx_s, String> {
        return Ok(wrapper::lgw_conf_channel_tx_s {
            enable: self.enable,
        });
    }
}

/// Structure containing the metadata of a packet that was received and a pointer to the payload.
pub struct RxPacket {
    /// Central frequency of the IF chain.
    pub freq_hz: u32,
    /// By which IF chain was packet received.
    pub channel: u8,
    /// Ctatus of the received packet.
    pub status: CRC,
    /// Internal concentrator counter for timestamping, 1 microsecond resolution.
    pub count_us: u32,
    /// Frequency error in Hz.
    pub freq_offset_hz: i32,
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
    /// Payload size in bytes.
    pub size: u16,
    /// Buffer containing the payload.
    pub payload: [u8; 256],
}

impl RxPacket {
    fn from_hal(pkt: wrapper::lgw_pkt_rx_s) -> Self {
        RxPacket {
            freq_hz: pkt.freq_hz,
            channel: pkt.channel,
            status: CRC::from_hal(pkt.status),
            count_us: pkt.count_us,
            freq_offset_hz: pkt.foff_hz,
            modulation: Modulation::from_hal(pkt.modulation),
            bandwidth: Bandwidth::from_hal(pkt.bandwidth),
            datarate: DataRate::from_hal(pkt.datarate),
            coderate: CodeRate::from_hal(pkt.coderate),
            rssi: pkt.rssi,
            snr: pkt.snr,
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
    /// TX power, in dBm.
    pub rf_power: i8,
    /// Modulation bandwidth (LoRa only).
    pub bandwidth: Bandwidth,
    /// TX datarate (SF for LoRa).
    pub datarate: DataRate,
    /// Error-correcting code of the packet (LoRa only).
    pub coderate: CodeRate,
    /// Invert signal polarity, for orthogonal downlinks (LoRa only).
    pub invert_pol: bool,
    /// Set the preamble length, 0 for default.
    pub preamble: u16,
    /// Public:0x21, Private:0x12.
    pub sync_word: u8,
    /// If true, do not send a CRC in the packet.
    pub no_crc: bool,
    /// If true, enable implicit header mode (LoRa).
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
            rf_power: 0,
            bandwidth: 0,
            datarate: DataRate::SF5,
            coderate: CodeRate::LoRa4_5,
            invert_pol: false,
            preamble: 0,
            sync_word: 0x21,
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
            rf_power: self.rf_power,
            bandwidth: self.bandwidth.to_hal(),
            datarate: self.datarate.to_hal(),
            coderate: self.coderate.to_hal(),
            invert_pol: self.invert_pol,
            preamble: self.preamble,
            sync_word: self.sync_word,
            no_crc: self.no_crc,
            no_header: self.no_header,
            size: self.size,
            payload: self.payload,
        }
    }
}

/// Configure the gateway board.
pub fn board_setconf(conf: &BoardConfig) -> Result<(), String> {
    let mut conf = conf.to_hal()?;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_board_setconf(&mut conf) };
    if ret != 0 {
        return Err("lgw_board_setconf failed".to_string());
    }

    return Ok(());
}

/// Configure a RX channel.
pub fn channel_rx_setconf(chan: u8, conf: &ChannelRxConfig) -> Result<(), String> {
    let mut conf = conf.to_hal()?;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_channel_rx_setconf(chan, &mut conf) };
    if ret != 0 {
        return Err("lgw_channel_rx_setconf failed".to_string());
    }

    return Ok(());
}

/// Configure TX.
pub fn channel_tx_setconf(conf: &ChannelTxConfig) -> Result<(), String> {
    let mut conf = conf.to_hal()?;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_channel_tx_setconf(&mut conf) };
    if ret != 0 {
        return Err("lgw_channel_tx_setconf failed".to_string());
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

/// A non-blocking function that will fetch packets from the LoRa concentrator FIFO
/// and data buffer.
pub fn receive() -> Result<Vec<RxPacket>, String> {
    let mut packets: [wrapper::lgw_pkt_rx_s; 8] = [Default::default(); 8];

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_receive(8, packets.as_mut_ptr()) };
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

    let ret = unsafe { wrapper::lgw_send(&pkt) };
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

/// Return instateneous value of internal counter.
pub fn get_instcnt() -> Result<u32, String> {
    let mut cnt = 0;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_get_instcnt(&mut cnt) };
    if ret != 0 {
        return Err("lgw_get_instcnt failed".to_string());
    }

    return Ok(cnt);
}

/// Allow user to check the version/options of the library once compiled.
pub fn version_info() -> String {
    unsafe {
        CStr::from_ptr(wrapper::lgw_version_info())
            .to_string_lossy()
            .into_owned()
    }
}

/// Return the LoRa concentrator EUI.
pub fn get_eui() -> Result<[u8; 8], String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let mut eui: u64 = 0;
    let ret = unsafe { wrapper::lgw_get_eui(&mut eui) };
    if ret != 0 {
        return Err("lgw_get_eui failed".to_string());
    }

    let eui = unsafe { transmute(eui.to_be()) };
    return Ok(eui);
}

/// Return the temperature measured by the LoRa concentrator sensor (updated every 30s).
pub fn get_temperature(source: TemperatureSource) -> Result<f32, String> {
    let mut temp: f32 = 0.0;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_get_temperature(&mut temp, &mut source.to_hal()) };
    if ret != 0 {
        return Err("lgw_get_temperature failed".to_string());
    }

    return Ok(temp);
}

/// Return time on air of given packet, in milliseconds.
pub fn time_on_air(pkt: &TxPacket) -> Result<Duration, String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let mut pkt = pkt.to_hal();
    let mut result: f64 = 0.0;

    let ms = unsafe { wrapper::lgw_time_on_air(&mut pkt, &mut result) };
    return Ok(Duration::from_millis(ms as u64));
}
