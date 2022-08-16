mod en_300_220;

use std::fmt;
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
pub enum Standard {
    None,
    En300_220,
}
impl Standard {
    pub fn from_str(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Ok(Standard::None);
        }
        match input {
            "EN_300_220" => Ok(Standard::En300_220),
            _ => Err(format!("Standard [{}] is unknown or not supported", input).to_owned()),
        }
    }

    pub fn get_configuration(&self) -> Configuration {
        match self {
            Self::En300_220 => en_300_220::get_configuration(),
            _ => Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Band {
    pub label: String,
    pub frequency_min: u32,
    pub frequency_max: u32,
    pub duty_cycle_percent_max: f32,
    pub tx_power_max: i8,
}
impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn freq_u32_to_f32(input: u32) -> f32 {
            return (input / 1000) as f32 / 1000.0;
        }
        write!(
            f,
            "{}:{:.2}-{:.2}",
            self.label,
            freq_u32_to_f32(self.frequency_min),
            freq_u32_to_f32(self.frequency_max)
        )
    }
}

#[derive(Default)]
pub struct Configuration {
    pub bands: Vec<Band>,
    pub window_time: Duration,
}
