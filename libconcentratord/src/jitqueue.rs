use std::time::Duration;

use log::{debug, info};

#[derive(PartialEq, Eq, Debug)]
pub enum EnqueueError {
    Unknown(String),
    Collision,
    FullQueue,
    TooLate,
    TooEarly,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TxMode {
    Immediate,
    Timestamped,
    OnGPS,
}

pub trait TxPacket {
    fn get_time_on_air(&self) -> Result<Duration, String>;
    fn get_tx_mode(&self) -> TxMode;
    fn get_id(&self) -> String;
    fn set_tx_mode(&mut self, tx_mode: TxMode);
    fn get_count_us(&self) -> u32;
    fn set_count_us(&mut self, count_us: u32);
}

pub struct Item<T> {
    pre_delay: Duration,
    post_delay: Duration,
    packet: T,
}

pub struct Queue<T> {
    items: Vec<Item<T>>,

    tx_start_delay: Duration,
    tx_margin_delay: Duration,
    tx_jit_delay: Duration,
    tx_max_advance_delay: Duration,
}

impl<T: TxPacket + Copy> Queue<T> {
    pub fn new(capacity: usize) -> Queue<T> {
        info!("Initializing JIT queue, capacity: {}", capacity);

        Queue {
            items: Vec::with_capacity(capacity),

            tx_start_delay: Duration::from_micros(1500),
            tx_margin_delay: Duration::from_micros(1000),
            tx_jit_delay: Duration::from_micros(30000),
            tx_max_advance_delay: Duration::from_secs((3 + 1) * 128),
        }
    }

    pub fn size(&self) -> usize {
        self.items.capacity()
    }

    pub fn empty(&self) -> bool {
        self.items.len() == 0
    }

    pub fn full(&self) -> bool {
        self.items.len() == self.size()
    }

    pub fn pop(&mut self, concentrator_count: u32) -> Option<T> {
        match self.items.first() {
            None => {
                // nothing in the queue
                return None;
            }
            Some(v) => {
                if v.packet.get_count_us().wrapping_sub(concentrator_count)
                    > v.pre_delay.as_micros() as u32
                {
                    // packet is too far in advance
                    return None;
                }
            }
        };

        let item = self.items.remove(0);

        return Some(item.packet);
    }

    pub fn enqueue(&mut self, concentrator_count: u32, packet: T) -> Result<(), EnqueueError> {
        match packet.get_tx_mode() {
            TxMode::Timestamped => {
                info!(
                    "Enqueueing timestamped packet, downlink_id: {}, counter_us: {}, current_counter_us: {}",
                    packet.get_id(),
                    packet.get_count_us(),
                    concentrator_count,
                );
            }
            TxMode::Immediate => {
                info!(
                    "Enqueueing immediate packet, downlink_id: {}, current_counter_us: {}",
                    packet.get_id(),
                    concentrator_count,
                );
            }
            TxMode::OnGPS => {}
        }

        if self.full() {
            return Err(EnqueueError::FullQueue);
        }

        let time_on_air = match packet.get_time_on_air() {
            Ok(v) => v,
            Err(err) => return Err(EnqueueError::Unknown(err)),
        };

        let mut item = Item {
            pre_delay: self.tx_start_delay + self.tx_jit_delay,
            post_delay: time_on_air,
            packet: packet,
        };

        // An immediate downlink becomes a timestamped downlink "ASAP".
        // Set the packet count_us to the first available slot.
        if item.packet.get_tx_mode() == TxMode::Immediate {
            item.packet.set_tx_mode(TxMode::Timestamped);

            // use now + 1 sec
            let mut asap_count_us =
                concentrator_count.wrapping_add(Duration::from_secs(1).as_micros() as u32);

            // check if there is a collision
            if self.collision_test(asap_count_us, item.pre_delay, item.post_delay) {
                for p in self.items.iter() {
                    asap_count_us = p.packet.get_count_us().wrapping_add(
                        (p.post_delay.as_micros()
                            + item.pre_delay.as_micros()
                            + self.tx_margin_delay.as_micros()) as u32,
                    );

                    if !self.collision_test(asap_count_us, item.pre_delay, item.post_delay) {
                        break;
                    }
                }
            }

            item.packet.set_count_us(asap_count_us);
        } else if item.packet.get_tx_mode() == TxMode::Timestamped {
            if self.collision_test(item.packet.get_count_us(), item.pre_delay, item.post_delay) {
                return Err(EnqueueError::Collision);
            }
        } else if item.packet.get_tx_mode() == TxMode::OnGPS {
            return Err(EnqueueError::Unknown("TODO".to_string()));
        }

        // Is it too late to send this packet?
        if item.packet.get_count_us().wrapping_sub(concentrator_count)
            < (self.tx_start_delay + self.tx_margin_delay + self.tx_jit_delay).as_micros() as u32
        {
            return Err(EnqueueError::TooLate);
        }

        // Is it too early to send this packet?
        if item.packet.get_count_us().wrapping_sub(concentrator_count)
            > self.tx_max_advance_delay.as_micros() as u32
        {
            return Err(EnqueueError::TooEarly);
        }

        debug!(
            "Packet enqueued, downlink_id: {}, count_us: {}",
            item.packet.get_id(),
            item.packet.get_count_us()
        );

        self.items.push(item);
        self.sort(concentrator_count);

        return Ok(());
    }

    fn sort(&mut self, count_us: u32) {
        self.items.sort_by(|a, b| {
            let a_diff = a.packet.get_count_us().wrapping_sub(count_us);
            let b_diff = b.packet.get_count_us().wrapping_sub(count_us);

            return a_diff.cmp(&b_diff);
        })
    }

    fn collision_test(&self, count_us: u32, pre_delay: Duration, post_delay: Duration) -> bool {
        let pre_delay = pre_delay.as_micros() as u32;
        let post_delay = post_delay.as_micros() as u32;

        for p2 in self.items.iter() {
            let p2_pre_delay = p2.pre_delay.as_micros() as u32;
            let p2_post_delay = p2.post_delay.as_micros() as u32;

            if ((count_us.wrapping_sub(p2.packet.get_count_us()))
                <= (pre_delay + p2_post_delay + self.tx_margin_delay.as_micros() as u32))
                || ((p2.packet.get_count_us().wrapping_sub(count_us))
                    <= (p2_pre_delay + post_delay + self.tx_margin_delay.as_micros() as u32))
            {
                return true;
            }
        }

        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone)]
    struct TxPacketMock {
        time_on_air: Duration,
        tx_mode: TxMode,
        count_us: u32,
    }

    impl TxPacket for TxPacketMock {
        fn get_time_on_air(&self) -> Result<Duration, String> {
            return Ok(self.time_on_air);
        }

        fn get_tx_mode(&self) -> TxMode {
            return self.tx_mode;
        }

        fn get_id(&self) -> String {
            return "".to_string();
        }

        fn set_tx_mode(&mut self, tx_mode: TxMode) {
            self.tx_mode = tx_mode;
        }

        fn get_count_us(&self) -> u32 {
            return self.count_us;
        }

        fn set_count_us(&mut self, count_us: u32) {
            self.count_us = count_us;
        }
    }

    #[test]
    fn test_size() {
        let q: Queue<TxPacketMock> = Queue::new(10);
        assert_eq!(10, q.size());
    }

    #[test]
    fn test_enqueue_full() {
        let mut q: Queue<TxPacketMock> = Queue::new(2);

        q.enqueue(
            100,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
            },
        )
        .unwrap();

        q.enqueue(
            100,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
            },
        )
        .unwrap();

        assert!(
            q.enqueue(
                100,
                TxPacketMock {
                    time_on_air: Duration::from_millis(100),
                    tx_mode: TxMode::Immediate,
                    count_us: 0,
                },
            )
            .is_err(),
            "jit queue should be full"
        );
    }

    #[test]
    fn test_enqueue_immediate() {
        let mut q: Queue<TxPacketMock> = Queue::new(2);
        let concentrator_count = 100;

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
            },
        )
        .unwrap();

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
            },
        )
        .unwrap();

        // first item is schedule 1s after concentrator_count.
        let item = &q.items[0];
        assert_eq!(TxMode::Timestamped, item.packet.get_tx_mode());
        assert_eq!(Duration::from_micros(1500 + 30000), item.pre_delay);
        assert_eq!(Duration::from_millis(100), item.post_delay);
        assert_eq!(
            concentrator_count + Duration::from_secs(1).as_micros() as u32,
            item.packet.get_count_us()
        );

        // second item is scheduled after 1st (taking into account the margins).
        let first_end_us = item.packet.get_count_us() + item.post_delay.as_micros() as u32;

        let item = &q.items[1];
        assert_eq!(TxMode::Timestamped, item.packet.get_tx_mode());
        assert_eq!(Duration::from_micros(1500 + 30000), item.pre_delay);
        assert_eq!(Duration::from_millis(100), item.post_delay);
        assert_eq!(
            first_end_us + item.pre_delay.as_micros() as u32 + q.tx_margin_delay.as_micros() as u32,
            item.packet.get_count_us()
        );
    }

    #[test]
    fn test_enqueue_immediate_u32_wrapping() {
        let mut q: Queue<TxPacketMock> = Queue::new(2);
        let concentrator_count = 0_u32.wrapping_sub(
            (Duration::from_secs(1)
                + Duration::from_micros(1500 + 30000)
                + Duration::from_millis(100))
            .as_micros() as u32,
        );

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
            },
        )
        .unwrap();

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
            },
        )
        .unwrap();

        let item = &q.items[0];
        assert_eq!(4294835796, item.packet.get_count_us());

        let item = &q.items[1];
        assert_eq!(1000, item.packet.get_count_us());
    }

    #[test]
    fn test_pop_empty() {
        let mut q: Queue<TxPacketMock> = Queue::new(2);

        let item = q.pop(Duration::from_secs(1).as_micros() as u32);
        assert_eq!(true, item.is_none());
    }

    #[test]
    fn test_pop() {
        let mut q: Queue<TxPacketMock> = Queue::new(2);
        let concentrator_count = Duration::from_secs(1).as_micros() as u32;

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Timestamped,
                count_us: Duration::from_secs(2).as_micros() as u32,
            },
        )
        .unwrap();

        let item = q.pop(Duration::from_secs(2).as_micros() as u32);
        assert_eq!(false, item.is_none());
    }

    #[test]
    fn test_pop_too_far_in_future() {
        let mut q: Queue<TxPacketMock> = Queue::new(2);
        let concentrator_count = Duration::from_secs(1).as_micros() as u32;

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Timestamped,
                count_us: Duration::from_secs(2).as_micros() as u32,
            },
        )
        .unwrap();

        let item = q.pop(Duration::from_secs(1).as_micros() as u32);
        assert_eq!(true, item.is_none());
    }

    #[test]
    fn test_pop_u32_wrapping() {
        let mut q: Queue<TxPacketMock> = Queue::new(2);
        let concentrator_count = 0_u32.wrapping_sub(Duration::from_secs(1).as_micros() as u32);

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Timestamped,
                count_us: 1,
            },
        )
        .unwrap();

        let item = q.pop(0_u32.wrapping_sub(100));
        assert_eq!(true, item.is_some());
    }
}
