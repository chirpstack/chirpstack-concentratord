use std::fmt;
use std::time::Duration;

use anyhow::Result;
use chirpstack_api::common::Regulation;
use serde::{Deserialize, Serialize};

use crate::error::Error;

mod etsi_en_300_220;

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Standard {
    ETSI_EN_300_220,
}

impl fmt::Display for Standard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct Band {
    pub label: String,
    pub frequency_min: u32,
    pub frequency_max: u32,
    pub duty_cycle_permille_max: u32,
    pub tx_power_max_eirp: i8,
}

impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[label: {}, freq_min: {}, freq_max: {}, dc_max: {:.2}%]",
            self.label,
            self.frequency_min,
            self.frequency_max,
            self.duty_cycle_permille_max as f32 / 10.0,
        )
    }
}

pub struct Configuration {
    pub bands: Vec<Band>,
    pub window_time: Duration,
    regulation: Regulation,
}

impl Configuration {
    pub fn new(s: Standard) -> Self {
        match s {
            Standard::ETSI_EN_300_220 => etsi_en_300_220::new(),
        }
    }

    pub fn get_band(&self, tx_freq: u32, tx_power_eirp: i8) -> Result<Band, Error> {
        for b in &self.bands {
            if b.frequency_min <= tx_freq
                && tx_freq < b.frequency_max
                && tx_power_eirp <= b.tx_power_max_eirp
            {
                return Ok(b.clone());
            }
        }

        Err(Error::BandNotFound(tx_freq, tx_power_eirp))
    }

    pub fn get_regulation(&self) -> Regulation {
        self.regulation
    }
}

pub fn get(s: Standard) -> Configuration {
    match s {
        Standard::ETSI_EN_300_220 => etsi_en_300_220::new(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_band() {
        struct Test {
            name: String,
            freq: u32,
            tx_power_eirp: i8,
            expected_band: Option<Band>,
        }

        let tests = vec![Test {
            name: "M band".into(),
            freq: 868100000,
            tx_power_eirp: 16,
            expected_band: Some(Band {
                label: "M".into(),
                frequency_min: 868000000,
                frequency_max: 868600000,
                duty_cycle_permille_max: 10,
                tx_power_max_eirp: 16,
            }),
        }];

        let c = Configuration::new(Standard::ETSI_EN_300_220);
        for tst in &tests {
            println!("> {}", tst.name);

            let res = c.get_band(tst.freq, tst.tx_power_eirp);
            if tst.expected_band.is_none() {
                assert!(res.is_err());
            } else {
                assert_eq!(tst.expected_band.as_ref().unwrap(), res.as_ref().unwrap());
            }
        }
    }
}
