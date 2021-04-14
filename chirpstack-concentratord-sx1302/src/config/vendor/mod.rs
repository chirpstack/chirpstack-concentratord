use libloragw_sx1302::hal;

pub mod rak;
pub mod semtech;

#[derive(Clone)]
pub enum ComType {
    SPI,
    USB,
}

impl Default for ComType {
    fn default() -> Self {
        ComType::SPI
    }
}

#[derive(Default, Clone)]
pub struct Configuration {
    pub radio_count: usize,
    pub clock_source: u8,
    pub full_duplex: bool,
    pub lora_multi_sf_bandwidth: u32,
    pub radio_config: Vec<RadioConfig>,
    pub gps_tty_path: Option<String>,
    pub com_type: ComType,
    pub com_path: String,
    pub reset_pin: Option<u32>,
    pub power_en_pin: Option<u32>,
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
