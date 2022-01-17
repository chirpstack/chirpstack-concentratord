use std::cmp::Ordering;
use std::marker::PhantomData;
use std::time::Duration;

use crate::jitqueue;

#[derive(Copy, Clone)]
pub struct Item<T> {
    pub time: Duration,
    pub airtime: Duration,
    pub frequency: u32,
    pub tx_power: i8,
    pub aggregated: bool,
    phantom: PhantomData<T>,
}

impl<T: jitqueue::TxPacket + Copy> Item<T> {
    pub fn new(item: &jitqueue::Item<T>) -> Item<T> {
        Item {
            time: item.time,
            airtime: item.post_delay,
            frequency: item.packet.get_frequency(),
            tx_power: item.packet.get_tx_power(),
            aggregated: false,
            phantom: PhantomData,
        }
    }
}

impl<T: jitqueue::TxPacket + Copy> Ord for Item<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl<T: jitqueue::TxPacket + Copy> PartialOrd for Item<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: jitqueue::TxPacket + Copy> PartialEq for Item<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl<T: jitqueue::TxPacket + Copy> Eq for Item<T> {}
