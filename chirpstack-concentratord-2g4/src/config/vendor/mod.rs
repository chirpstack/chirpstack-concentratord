pub mod multitech;
pub mod rak;
pub mod semtech;

#[derive(Default, Clone)]
pub struct Configuration {
    pub tty_path: String,
    pub tx_min_max_freqs: Vec<(u32, u32)>,
    pub reset_pin: Option<(String, u32)>,
    pub boot0_pin: Option<(String, u32)>,
}
