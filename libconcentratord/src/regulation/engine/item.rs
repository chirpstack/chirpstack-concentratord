use std::cmp::Ordering;
use std::convert::From;
use std::time::Duration;

use crate::jitqueue;

#[derive(Copy, Clone)]
pub struct Item {
    pub time: Duration,
    pub airtime: Duration,
    pub frequency: u32,
    pub tx_power: i8,
    pub aggregated: bool,
}

impl<T: jitqueue::TxPacket + Copy> From<&jitqueue::Item<T>> for Item {
    fn from(item: &jitqueue::Item<T>) -> Self {
        Item {
            time: item.time,
            airtime: item.post_delay,
            frequency: item.packet.get_frequency(),
            tx_power: item.packet.get_tx_power(),
            aggregated: false,
        }
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for Item {}
