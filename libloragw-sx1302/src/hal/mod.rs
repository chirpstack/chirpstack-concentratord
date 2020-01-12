use std::ffi::CString;
use std::mem::transmute;
use std::os::raw::c_char;
use std::time::Duration;

use super::{mutex, wrapper};

#[derive(Debug, Copy, Clone)]
pub enum RadioType {
    NONE,
    SX1255,
    SX1257,
    SX1272,
    SX1276,
    SX1250,
}

#[derive(Debug, Copy, Clone)]
pub enum CRC {
    Undefined,
    NoCRC,
    BadCRC,
    CRCOk,
}

#[derive(Debug, Copy, Clone)]
pub enum Modulation {
    Undefined,
    LoRa,
    FSK,
}

#[derive(Debug, Copy, Clone)]
pub enum DataRate {
    Undefined,
    SF5,
    SF6,
    SF7,
    SF8,
    SF9,
    SF10,
    SF11,
    SF12,
    FSK(u32),
    FSKMin,
    FSKMax,
}

#[derive(Debug, Copy, Clone)]
pub enum CodeRate {
    Undefined,
    LoRa4_5,
    LoRa4_6,
    LoRa4_7,
    LoRa4_8,
}

#[derive(Debug, Copy, Clone)]
pub enum TxMode {
    Immediate,
    Timestamped,
    OnGPS,
}

#[derive(PartialEq, Eq)]
pub enum StatusSelect {
    Tx,
    Rx,
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

#[derive(Debug)]
pub enum RxStatus {
    Unknown,
    Off,
    On,
    Suspended,
}

/// Configuration structure for board specificities.
pub struct BoardConfig {
    /// Enable ONLY for *public* networks using the LoRa MAC protocol.
    pub lorawan_public: bool,
    /// Index of RF chain which provides clock to concentrator.
    pub clock_source: u8,
    /// Indicates if the gateway operates in full duplex mode or not.
    pub full_duplex: bool,
    /// Path to access the SPI device to connect to the SX1302.
    pub spidev_path: String,
}

/// Configuration structure for a RF chain.
pub struct RxRfConfig {
    /// Enable or disable that RF chain.
    pub enable: bool,
    /// Center frequency of the radio in Hz.
    pub freq_hz: u32,
    /// Board-specific RSSI correction factor.
    pub rssi_offset: f32,
    /// Board-specific RSSI temperature compensation coefficients.
    pub rssi_temp_compensation: RssiTempCompensationConfig,
    /// Radio type for that RF chain (SX1255, SX1257....).
    pub radio_type: RadioType,
    /// Enable or disable TX on that RF chain.
    pub tx_enable: bool,
    /// Configure the radio in single or differential input mode (SX1250 only).
    pub single_input_mode: bool,
}

/// Structure containing all coefficients necessary to compute the offset to be applied on RSSI for
/// current temperature.
#[derive(Clone, Copy)]
pub struct RssiTempCompensationConfig {
    pub coeff_a: f32,
    pub coeff_b: f32,
    pub coeff_c: f32,
    pub coeff_d: f32,
    pub coeff_e: f32,
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
    pub bandwidth: u32,
    /// RX datarate, 0 for default.
    pub datarate: DataRate,
    /// size of FSK sync word (number of bytes, 0 for default).
    pub sync_word_size: u8,
    /// FSK sync word (ALIGN RIGHT, eg. 0xC194C1).
    pub sync_word: u64,
    /// LoRa Service implicit header.
    pub implicit_header: bool,
    /// LoRa Service implicit header payload length (number of bytes, 0 for default).
    pub implicit_payload_length: u8,
    /// LoRa Service implicit header CRC enable.
    pub implicit_crc_enable: bool,
    /// LoRa Service implicit header coding rate.
    pub implicit_coderate: CodeRate,
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
            implicit_header: false,
            implicit_payload_length: 0,
            implicit_crc_enable: false,
            implicit_coderate: CodeRate::Undefined,
        }
    }
}

/// Structure containing all gains of Tx chain.
#[derive(Clone)]
pub struct TxGainConfig {
    /// Measured TX power at the board connector, in dBm.
    pub rf_power: i8,
    /// (sx125x) 2 bits: control of the digital gain of SX1302.
    pub dig_gain: u8,
    /// (sx125x) 2 bits: control of the external PA (SX1302 I/O).
    /// (sx1250) 1 bits: enable/disable the external PA (SX1302 I/O).
    pub pa_gain: u8,
    /// (sx125x) 2 bits: control of the radio DAC.
    pub dac_gain: u8,
    /// 4 bits: control of the radio mixer.
    pub mix_gain: u8,
    /// (sx125x) calibrated I offset.
    pub offset_i: i8,
    /// (sx125x) calibrated Q offset.
    pub offset_q: i8,
    /// (sx1250) 6 bits: control the radio power index to be used for configuration.
    pub pwr_idx: u8,
}

/// Structure containing the metadata of a packet that was received and a pointer to the payload.
pub struct RxPacket {
    /// Central frequency of the IF chain.
    pub freq_hz: u32,
    /// freq_offset.
    pub freq_offset: i32,
    /// By which IF chain was packet received.
    pub if_chain: u8,
    /// Ctatus of the received packet.
    pub status: CRC,
    /// Internal concentrator counter for timestamping, 1 microsecond resolution.
    pub count_us: u32,
    /// Through which RF chain the packet was received.
    pub rf_chain: u8,
    /// modem_id.
    pub modem_id: u8,
    /// Modulation used by the packet.
    pub modulation: Modulation,
    /// Modulation bandwidth (LoRa only).
    pub bandwidth: u32,
    /// RX datarate of the packet (SF for LoRa).
    pub datarate: DataRate,
    /// Error-correcting code of the packet (LoRa only).
    pub coderate: CodeRate,
    /// Average RSSI of the channel in dB.
    pub rssic: f32,
    /// Average RSSI of the signal in dB.
    pub rssis: f32,
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
    /// Frequency offset from Radio Tx frequency (CW mode).
    pub freq_offset: i8,
    /// Modulation bandwidth (LoRa only).
    pub bandwidth: u32,
    /// TX datarate (baudrate for FSK, SF for LoRa).
    pub datarate: DataRate,
    /// Error-correcting code of the packet (LoRa only).
    pub coderate: CodeRate,
    /// Onvert signal polarity, for orthogonal downlinks (LoRa only).
    pub invert_pol: bool,
    /// Frequency deviation, in kHz (FSK only).
    pub f_dev: u8,
    /// Set the preamble length, 0 for default.
    pub preamble: u16,
    /// If true, do not send a CRC in the packet.
    pub no_crc: bool,
    /// Of true, enable implicit header mode (LoRa), fixed length (FSK).
    pub no_header: bool,
    /// Payload size in bytes.
    pub size: u16,
    /// Buffer containing the payload.
    pub payload: [u8; 256],
}

/// Configuration structure for the timestamp.
pub struct TimestampConfig {
    pub enable_precision_ts: bool,
    pub max_ts_metrics: u8,
    pub nb_symbols: u8,
}

const MAX_PKT: usize = 8;

/// Configure the gateway board.
pub fn board_setconf(conf: &BoardConfig) -> Result<(), String> {
    let spidev_path = CString::new(conf.spidev_path.clone()).unwrap();
    let spidev_path = spidev_path.as_bytes_with_nul();
    if spidev_path.len() > 64 {
        return Err("spidev_path max length is 64".to_string());
    }
    let mut spidev_path_chars = [0; 64];
    for (i, b) in spidev_path.iter().enumerate() {
        spidev_path_chars[i] = *b as c_char;
    }
    let mut conf = wrapper::lgw_conf_board_s {
        lorawan_public: conf.lorawan_public,
        clksrc: conf.clock_source,
        full_duplex: conf.full_duplex,
        spidev_path: spidev_path_chars,
    };

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_board_setconf(&mut conf) };
    if ret != 0 {
        return Err("lgw_board_setconf failed".to_string());
    }

    return Ok(());
}

/// Configure an RF chain (must configure before start).
pub fn rxrf_setconf(rf_chain: u8, conf: &RxRfConfig) -> Result<(), String> {
    let mut conf = wrapper::lgw_conf_rxrf_s {
        enable: conf.enable,
        freq_hz: conf.freq_hz,
        rssi_offset: conf.rssi_offset,
        rssi_tcomp: wrapper::lgw_rssi_tcomp_s {
            coeff_a: conf.rssi_temp_compensation.coeff_a,
            coeff_b: conf.rssi_temp_compensation.coeff_b,
            coeff_c: conf.rssi_temp_compensation.coeff_c,
            coeff_d: conf.rssi_temp_compensation.coeff_d,
            coeff_e: conf.rssi_temp_compensation.coeff_e,
        },
        type_: match conf.radio_type {
            RadioType::NONE => wrapper::lgw_radio_type_t_LGW_RADIO_TYPE_NONE,
            RadioType::SX1255 => wrapper::lgw_radio_type_t_LGW_RADIO_TYPE_SX1255,
            RadioType::SX1257 => wrapper::lgw_radio_type_t_LGW_RADIO_TYPE_SX1257,
            RadioType::SX1272 => wrapper::lgw_radio_type_t_LGW_RADIO_TYPE_SX1272,
            RadioType::SX1276 => wrapper::lgw_radio_type_t_LGW_RADIO_TYPE_SX1276,
            RadioType::SX1250 => wrapper::lgw_radio_type_t_LGW_RADIO_TYPE_SX1250,
        },
        tx_enable: conf.tx_enable,
        single_input_mode: conf.single_input_mode,
    };

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_rxrf_setconf(rf_chain, &mut conf) };
    if ret != 0 {
        return Err("lgw_rxrf_setconf failed".to_string());
    }

    return Ok(());
}

/// Configure an IF chain + modem (must configure before start).
pub fn rxif_setconf(if_chain: u8, conf: &RxIfConfig) -> Result<(), String> {
    let mut conf = wrapper::lgw_conf_rxif_s {
        enable: conf.enable,
        rf_chain: conf.rf_chain,
        freq_hz: conf.freq_hz,
        bandwidth: map_bandwidth(conf.bandwidth),
        datarate: map_data_rate(conf.datarate),
        sync_word_size: conf.sync_word_size,
        sync_word: conf.sync_word,
        implicit_hdr: conf.implicit_header,
        implicit_payload_length: conf.implicit_payload_length,
        implicit_crc_en: conf.implicit_crc_enable,
        implicit_coderate: map_code_rate(conf.implicit_coderate),
        ..Default::default()
    };

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_rxif_setconf(if_chain, &mut conf) };
    if ret != 0 {
        return Err("lgw_rxif_setconf failed".to_string());
    }

    return Ok(());
}

/// Configure the Tx gain LUT.
pub fn txgain_setconf(rf_chain: u8, txgain: &[TxGainConfig]) -> Result<(), String> {
    let mut conf = wrapper::lgw_tx_gain_lut_s {
        lut: [wrapper::lgw_tx_gain_s {
            rf_power: 0,
            dig_gain: 0,
            pa_gain: 0,
            dac_gain: 0,
            mix_gain: 0,
            offset_i: 0,
            offset_q: 0,
            pwr_idx: 0,
        }; 16],
        size: txgain.len() as u8,
    };

    for (i, gain) in txgain.iter().enumerate() {
        conf.lut[i] = wrapper::lgw_tx_gain_s {
            rf_power: gain.rf_power,
            dig_gain: gain.dig_gain,
            pa_gain: gain.pa_gain,
            dac_gain: gain.dac_gain,
            mix_gain: gain.mix_gain,
            offset_i: gain.offset_i,
            offset_q: gain.offset_q,
            pwr_idx: gain.pwr_idx,
        };
    }

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_txgain_setconf(rf_chain, &mut conf) };
    if ret != 0 {
        return Err("lgw_txgain_setconf failed".to_string());
    }

    return Ok(());
}

/// Configure the precision timestamp.
pub fn timestamp_setconf(conf: &TimestampConfig) -> Result<(), String> {
    let mut conf = wrapper::lgw_conf_timestamp_s {
        enable_precision_ts: conf.enable_precision_ts,
        max_ts_metrics: conf.max_ts_metrics,
        nb_symbols: conf.nb_symbols,
    };

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_timestamp_setconf(&mut conf) };
    if ret != 0 {
        return Err("lgw_timestamp_setconf failed".to_string());
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

/// A non-blocking function that will fetch up to 'max_pkt' packets from the LoRa concentrator FIFO
/// and data buffer.
pub fn receive() -> Result<Vec<RxPacket>, String> {
    let mut packets: [wrapper::lgw_pkt_rx_s; MAX_PKT] = [wrapper::lgw_pkt_rx_s {
        freq_hz: 0,
        freq_offset: 0,
        if_chain: 0,
        status: 0,
        count_us: 0,
        rf_chain: 0,
        modem_id: 0,
        modulation: 0,
        bandwidth: 0,
        datarate: 0,
        coderate: 0,
        rssic: 0.0,
        rssis: 0.0,
        snr: 0.0,
        snr_min: 0.0,
        snr_max: 0.0,
        crc: 0,
        size: 0,
        payload: [0; 256],
    }; MAX_PKT];

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_receive(MAX_PKT as u8, packets.as_mut_ptr()) };
    if ret == -1 {
        return Err("lgw_receive failed".to_string());
    }

    let mut v: Vec<RxPacket> = Vec::new();

    for i in 0..ret {
        let pkt = packets[i as usize];

        v.push(RxPacket {
            freq_hz: pkt.freq_hz,
            freq_offset: pkt.freq_offset,
            if_chain: pkt.if_chain,
            status: unmap_status(pkt.status),
            count_us: pkt.count_us,
            rf_chain: pkt.rf_chain,
            modem_id: pkt.modem_id,
            modulation: unmap_modulation(pkt.modulation),
            bandwidth: unmap_bandwidth(pkt.bandwidth),
            datarate: unmap_data_rate(pkt.datarate),
            coderate: unmap_code_rate(pkt.coderate),
            rssic: pkt.rssic,
            rssis: pkt.rssis,
            snr: pkt.snr,
            snr_min: pkt.snr_min,
            snr_max: pkt.snr_max,
            crc: pkt.crc,
            size: pkt.size,
            payload: pkt.payload,
        });
    }

    return Ok(v);
}

/// Schedule a packet to be send immediately or after a delay depending on tx_mode.
///
///
/// When sending a packet, there is a delay (approx 1.5ms) for the analog
/// circuitry to start and be stable. This delay is adjusted by the HAL depending
/// on the board version (lgw_i_tx_start_delay_us).
/// In 'timestamp' mode, this is transparent: the modem is started
/// lgw_i_tx_start_delay_us microseconds before the user-set timestamp value is
/// reached, the preamble of the packet start right when the internal timestamp
/// counter reach target value.
/// In 'immediate' mode, the packet is emitted as soon as possible: transferring the
/// packet (and its parameters) from the host to the concentrator takes some time,
/// then there is the lgw_i_tx_start_delay_us, then the packet is emitted.
/// In 'triggered' mode (aka PPS/GPS mode), the packet, typically a beacon, is
/// emitted lgw_i_tx_start_delay_us microsenconds after a rising edge of the
/// trigger signal. Because there is no way to anticipate the triggering event and
/// start the analog circuitry beforehand, that delay must be taken into account in
/// the protocol.
pub fn send(pkt: &TxPacket) -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let mut pkt = map_tx_packet(pkt);
    let ret = unsafe { wrapper::lgw_send(&mut pkt) };
    if ret != 0 {
        return Err("lgw_send failed".to_string());
    }
    return Ok(());
}

/// Give the the status of different part of the LoRa concentrator.
pub fn status(rf_chain: u8, select: StatusSelect) -> Result<StatusReturn, String> {
    let mut code = 0;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe {
        wrapper::lgw_status(
            rf_chain,
            match select {
                StatusSelect::Tx => wrapper::TX_STATUS,
                StatusSelect::Rx => wrapper::RX_STATUS,
            } as u8,
            &mut code,
        )
    };
    if ret != 0 {
        return Err("lgw_status failed".to_string());
    }

    if select == StatusSelect::Tx {
        return Ok(StatusReturn::Tx(match code as u32 {
            wrapper::TX_OFF => TxStatus::Off,
            wrapper::TX_FREE => TxStatus::Free,
            wrapper::TX_SCHEDULED => TxStatus::Scheduled,
            wrapper::TX_EMITTING => TxStatus::Emitting,
            _ => TxStatus::Unknown,
        }));
    } else {
        return Ok(StatusReturn::Rx(match code as u32 {
            wrapper::RX_OFF => RxStatus::Off,
            wrapper::RX_ON => RxStatus::On,
            wrapper::RX_SUSPENDED => RxStatus::Suspended,
            _ => RxStatus::Unknown,
        }));
    }
}

/// Abort a currently scheduled or ongoing TX.
pub fn abort_tx(rf_chain: u8) -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_abort_tx(rf_chain) };
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

/// Return the temperature measured by the LoRa concentrator sensor.
pub fn get_temperature() -> Result<f32, String> {
    let mut temp: f32 = 0.0;

    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ret = unsafe { wrapper::lgw_get_temperature(&mut temp) };
    if ret != 0 {
        return Err("lgw_get_temperature failed".to_string());
    }

    return Ok(temp);
}

/// Return time on air of given packet, in milliseconds.
pub fn time_on_air(pkt: &TxPacket) -> Result<Duration, String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let ms = unsafe { wrapper::lgw_time_on_air(&mut map_tx_packet(pkt)) };
    return Ok(Duration::from_millis(ms as u64));
}

fn map_tx_packet(pkt: &TxPacket) -> wrapper::lgw_pkt_tx_s {
    return wrapper::lgw_pkt_tx_s {
        freq_hz: pkt.freq_hz,
        tx_mode: map_tx_mode(pkt.tx_mode),
        count_us: pkt.count_us,
        rf_chain: pkt.rf_chain,
        rf_power: pkt.rf_power,
        modulation: map_modulation(pkt.modulation),
        freq_offset: pkt.freq_offset,
        bandwidth: map_bandwidth(pkt.bandwidth),
        datarate: map_data_rate(pkt.datarate),
        coderate: map_code_rate(pkt.coderate),
        invert_pol: pkt.invert_pol,
        f_dev: pkt.f_dev,
        preamble: pkt.preamble,
        no_crc: pkt.no_crc,
        no_header: pkt.no_header,
        size: pkt.size,
        payload: pkt.payload,
    };
}

fn map_bandwidth(bandwidth: u32) -> u8 {
    return match bandwidth {
        500000 => wrapper::BW_500KHZ,
        250000 => wrapper::BW_250KHZ,
        125000 => wrapper::BW_125KHZ,
        _ => wrapper::BW_UNDEFINED,
    } as u8;
}

fn map_code_rate(coderate: CodeRate) -> u8 {
    return match coderate {
        CodeRate::Undefined => wrapper::CR_UNDEFINED,
        CodeRate::LoRa4_5 => wrapper::CR_LORA_4_5,
        CodeRate::LoRa4_6 => wrapper::CR_LORA_4_6,
        CodeRate::LoRa4_7 => wrapper::CR_LORA_4_7,
        CodeRate::LoRa4_8 => wrapper::CR_LORA_4_8,
    } as u8;
}

fn map_data_rate(datarate: DataRate) -> u32 {
    return match datarate {
        DataRate::Undefined => wrapper::DR_UNDEFINED,
        DataRate::SF5 => wrapper::DR_LORA_SF5,
        DataRate::SF6 => wrapper::DR_LORA_SF6,
        DataRate::SF7 => wrapper::DR_LORA_SF7,
        DataRate::SF8 => wrapper::DR_LORA_SF8,
        DataRate::SF9 => wrapper::DR_LORA_SF9,
        DataRate::SF10 => wrapper::DR_LORA_SF10,
        DataRate::SF11 => wrapper::DR_LORA_SF11,
        DataRate::SF12 => wrapper::DR_LORA_SF12,
        DataRate::FSK(v) => v,
        DataRate::FSKMin => wrapper::DR_FSK_MIN,
        DataRate::FSKMax => wrapper::DR_FSK_MAX,
    } as u32;
}

fn unmap_status(status: u8) -> CRC {
    return match status as u32 {
        wrapper::STAT_NO_CRC => CRC::NoCRC,
        wrapper::STAT_CRC_BAD => CRC::BadCRC,
        wrapper::STAT_CRC_OK => CRC::CRCOk,
        _ => CRC::Undefined,
    };
}

fn map_modulation(modulation: Modulation) -> u8 {
    return match modulation {
        Modulation::Undefined => wrapper::MOD_UNDEFINED,
        Modulation::LoRa => wrapper::MOD_LORA,
        Modulation::FSK => wrapper::MOD_FSK,
    } as u8;
}

fn unmap_modulation(modulation: u8) -> Modulation {
    return match modulation as u32 {
        wrapper::MOD_LORA => Modulation::LoRa,
        wrapper::MOD_FSK => Modulation::FSK,
        _ => Modulation::Undefined,
    };
}

fn unmap_bandwidth(bandwidth: u8) -> u32 {
    return match bandwidth as u32 {
        wrapper::BW_500KHZ => 500000,
        wrapper::BW_250KHZ => 250000,
        wrapper::BW_125KHZ => 125000,
        _ => 0,
    };
}

fn unmap_data_rate(datarate: u32) -> DataRate {
    return match datarate {
        wrapper::DR_UNDEFINED => DataRate::Undefined,
        wrapper::DR_LORA_SF5 => DataRate::SF5,
        wrapper::DR_LORA_SF6 => DataRate::SF6,
        wrapper::DR_LORA_SF7 => DataRate::SF7,
        wrapper::DR_LORA_SF8 => DataRate::SF8,
        wrapper::DR_LORA_SF9 => DataRate::SF9,
        wrapper::DR_LORA_SF10 => DataRate::SF10,
        wrapper::DR_LORA_SF11 => DataRate::SF11,
        wrapper::DR_LORA_SF12 => DataRate::SF12,
        _ => DataRate::FSK(datarate),
    };
}

fn unmap_code_rate(coderate: u8) -> CodeRate {
    return match coderate as u32 {
        wrapper::CR_LORA_4_5 => CodeRate::LoRa4_5,
        wrapper::CR_LORA_4_6 => CodeRate::LoRa4_6,
        wrapper::CR_LORA_4_7 => CodeRate::LoRa4_7,
        wrapper::CR_LORA_4_8 => CodeRate::LoRa4_8,
        _ => CodeRate::Undefined,
    };
}

fn map_tx_mode(tx_mode: TxMode) -> u8 {
    return match tx_mode {
        TxMode::Immediate => wrapper::IMMEDIATE,
        TxMode::Timestamped => wrapper::TIMESTAMPED,
        TxMode::OnGPS => wrapper::ON_GPS,
    } as u8;
}
