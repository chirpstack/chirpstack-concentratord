use super::{mutex, wrapper};
use std::ffi::CString;

/// Communication type.
#[derive(Debug, PartialEq)]
pub enum ComType {
    SPI,
    USB,
    UNKNOWN,
}

impl ComType {
    pub fn to_hal(&self) -> u32 {
        match self {
            ComType::SPI => wrapper::com_type_e_LGW_COM_SPI,
            ComType::USB => wrapper::com_type_e_LGW_COM_USB,
            ComType::UNKNOWN => wrapper::com_type_e_LGW_COM_UNKNOWN,
        }
    }
}

pub fn open(com_type: ComType, com_path: &str) -> Result<(), String> {
    let _guard = mutex::CONCENTATOR.lock().unwrap();

    let com_type = com_type.to_hal();
    let com_path = CString::new(com_path).unwrap();

    let ret = unsafe { wrapper::lgw_com_open(com_type, com_path.into_raw()) };
    if ret != 0 {
        return Err("lgw_com_open failed".to_string());
    }

    return Ok(());
}
