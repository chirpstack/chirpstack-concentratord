use std::time::Duration;

pub trait ToConcentratorCount {
    fn to_concentrator_count(self) -> u32;
}

impl ToConcentratorCount for Duration {
    fn to_concentrator_count(self) -> u32 {
        (self.as_micros() % (1 << 32)) as u32
    }
}
