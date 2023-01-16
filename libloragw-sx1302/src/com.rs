use std::ffi::CString;

use anyhow::Result;

use super::{mutex, wrapper};

/// Communication type.
#[derive(Debug, PartialEq)]
pub enum ComType {
    Spi,
    Usb,
    Unknown,
}

impl ComType {
    pub fn to_hal(&self) -> u32 {
        match self {
            ComType::Spi => wrapper::com_type_e_LGW_COM_SPI,
            ComType::Usb => wrapper::com_type_e_LGW_COM_USB,
            ComType::Unknown => wrapper::com_type_e_LGW_COM_UNKNOWN,
        }
    }
}

pub fn open(com_type: ComType, com_path: &str) -> Result<()> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();

    let com_type = com_type.to_hal();
    let com_path = CString::new(com_path).unwrap();

    let ret = unsafe { wrapper::lgw_com_open(com_type, com_path.into_raw()) };
    if ret != 0 {
        return Err(anyhow!("lgw_com_open failed"));
    }

    Ok(())
}
