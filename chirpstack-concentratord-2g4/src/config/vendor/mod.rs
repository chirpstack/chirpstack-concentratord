pub mod semtech;

#[derive(Default, Clone)]
pub struct Configuration {
    pub tty_path: String,
    pub min_max_tx_freq: (u32, u32),
    pub reset_pin: Option<u32>,
    pub boot0_pin: Option<u32>,
}
