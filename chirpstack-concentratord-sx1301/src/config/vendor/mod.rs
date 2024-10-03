use libconcentratord::gnss;
use libloragw_sx1301::hal;

pub mod imst;
pub mod kerlink;
pub mod multitech;
pub mod pi_supply;
pub mod rak;
pub mod risinghf;
pub mod sandbox;
pub mod wifx;

#[derive(Default, Clone)]
pub struct Configuration {
    pub radio_count: usize,
    pub clock_source: u8,
    pub radio_rssi_offset: Vec<f32>,
    pub radio_tx_enabled: Vec<bool>,
    pub radio_type: Vec<hal::RadioType>,
    pub tx_min_max_freqs: Vec<(u32, u32)>,
    pub radio_tx_notch_freq: Vec<u32>,
    pub lora_multi_sf_bandwidth: u32,
    pub tx_gain_table: Vec<hal::TxGainConfig>,
    pub gps: gnss::Device,
    pub spidev_path: String,
    pub reset_pin: Option<(String, u32)>,
    pub enforce_duty_cycle: bool,
}
