use std::ffi::CString;

use anyhow::Result;

use super::{mutex, wrapper};

/// Set SPI device.
pub fn set_path(spidev_path: &str) -> Result<()> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();
    let spidev_path = CString::new(spidev_path).unwrap();
    let ret = unsafe { wrapper::lgw_spi_set_path(spidev_path.as_ptr()) };
    if ret != 0 {
        return Err(anyhow!("lgw_spi_set_path failed"));
    }

    Ok(())
}
