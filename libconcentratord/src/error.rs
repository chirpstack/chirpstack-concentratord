#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Item exceeds duty-cycle")]
    DutyCycle,

    #[error("Item would exceed duty-cycle with future items")]
    DutyCycleFutureItems,

    #[error("No band for freq: {0}, tx_power_eirp: {1}")]
    BandNotFound(u32, i8),

    #[error("Timeout")]
    Timeout,

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
