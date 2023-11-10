use libloragw_sx1302::hal;

pub mod dragino;
pub mod embit;
pub mod multitech;
pub mod rak;
pub mod seeed;
pub mod semtech;
pub mod waveshare;

#[derive(Default, Clone, PartialEq)]
pub enum Gps {
    #[default]
    None,
    TtyPath(String),
    Gpsd,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum ComType {
    #[default]
    Spi,
    Usb,
}

#[derive(Default, Clone)]
pub struct Configuration {
    pub radio_count: usize,
    pub clock_source: u8,
    pub full_duplex: bool,
    pub lora_multi_sf_bandwidth: u32,
    pub radio_config: Vec<RadioConfig>,
    pub gps: Gps,
    pub com_type: ComType,
    pub com_path: String,
    pub i2c_path: Option<String>,
    pub i2c_temp_sensor_addr: Option<u8>,
    pub sx1302_reset_pin: Option<(String, u32)>,
    pub sx1302_power_en_pin: Option<(String, u32)>,
    pub sx1261_reset_pin: Option<(String, u32)>,
    pub ad5338r_reset_pin: Option<(String, u32)>,
    pub reset_commands: Option<Vec<(String, Vec<String>)>>,
}

#[derive(Clone)]
pub struct RadioConfig {
    pub enable: bool,
    pub radio_type: hal::RadioType,
    pub single_input_mode: bool,
    pub rssi_offset: f32,
    pub rssi_temp_compensation: hal::RssiTempCompensationConfig,
    pub tx_enable: bool,
    pub tx_freq_min: u32,
    pub tx_freq_max: u32,
    pub tx_gain_table: Vec<hal::TxGainConfig>,
}
