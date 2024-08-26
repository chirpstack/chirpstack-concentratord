use std::time::Duration;

use anyhow::Result;
use chirpstack_api::gw;
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::helpers::ToConcentratorCount;
use crate::regulation::{dutycycle, tracker};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TxMode {
    Immediate,
    Timestamped,
    OnGPS,
}

pub trait TxPacket {
    fn get_time_on_air(&self) -> Result<Duration>;
    fn get_tx_mode(&self) -> TxMode;
    fn get_id(&self) -> u32;
    fn set_tx_mode(&mut self, tx_mode: TxMode);
    fn get_count_us(&self) -> u32;
    fn set_count_us(&mut self, count_us: u32);
    fn get_frequency(&self) -> u32;
    fn get_tx_power(&self) -> i8;
}

pub struct Item<T> {
    // This value is derived from the concentrator_count, but will always increment, instead of
    // periodically rollover as the concentrator_count does.
    linear_count: Duration,
    pre_delay: Duration,
    post_delay: Duration,
    packet: T,
}

pub struct Queue<T> {
    items: Vec<Item<T>>,
    dc_tracker: Option<tracker::Tracker>,

    tx_start_delay: Duration,
    tx_margin_delay: Duration,
    tx_jit_delay: Duration,
    tx_max_advance_delay: Duration,

    // The queue instance keeps track of the last concentrator_count and linear_count values in
    // order to convert a new concentrator_count value into a linear_count. Note that the
    // concentrator_count_last will rollover back to 0, the linear_count_last will always
    // increment.
    concentrator_count_last: u32,
    linear_count_last: Duration,

    // This value holds the linear counter value after finishing the last downlink transmission. We
    // need to store this as once the downlink is scheduled, it is popped from the queue and we no
    // longer know until when the concentrator is busy transmitting.
    tx_linear_count_finished: Duration,
}

impl<T: TxPacket + Copy> Queue<T> {
    pub fn new(capacity: usize, dc_tracker: Option<tracker::Tracker>) -> Queue<T> {
        info!("Initializing JIT queue, capacity: {}", capacity);

        Queue {
            dc_tracker,
            items: Vec::with_capacity(capacity),

            tx_start_delay: Duration::from_micros(1500),
            tx_margin_delay: Duration::from_micros(1000),
            tx_jit_delay: Duration::from_micros(40000),
            tx_max_advance_delay: Duration::from_secs((3 + 1) * 128),

            concentrator_count_last: 0,
            linear_count_last: Duration::from_secs(0),
            tx_linear_count_finished: Duration::from_secs(0),
        }
    }

    pub fn size(&self) -> usize {
        self.items.capacity()
    }

    pub fn empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn full(&self) -> bool {
        self.items.len() == self.size()
    }

    pub fn pop(&mut self, concentrator_count: u32) -> Option<T> {
        let linear_count = self.get_linear_count(concentrator_count);

        match self.items.first() {
            None => {
                // nothing in the queue
                return None;
            }
            Some(v) => {
                if v.linear_count < linear_count {
                    // it can happen if cpu load is too high but should normally
                    // not happen.
                    error!("Scheduled packet is too old, dropped: count_us: {}, current_counter_us: {}", 
                        v.packet.get_count_us(),
                        concentrator_count);
                    self.items.remove(0);
                    return None;
                }

                if v.linear_count - linear_count > v.pre_delay {
                    // packet is too far in advance
                    return None;
                }
            }
        };

        let item = self.items.remove(0);

        // This value holds the counter when the concentrator is done transmitting the packet. This
        // is needed to detect possible collisions if enqueueing new packets.
        self.tx_linear_count_finished = item.linear_count + item.post_delay;

        Some(item.packet)
    }

    pub fn get_duty_cycle_stats(&mut self, concentrator_count: u32) -> Option<gw::DutyCycleStats> {
        let linear_count = self.get_linear_count(concentrator_count);

        if let Some(dc_tracker) = &self.dc_tracker {
            let window = dc_tracker.get_window();
            let band_stats = dc_tracker
                .get_tracked_durations(linear_count)
                .iter()
                .map(|(band, duration)| {
                    info!(
                        "Duty-cyle stats: {} - current_dc: {:.2}%",
                        band,
                        duration.as_nanos() as f64 / window.as_nanos() as f64 * 100.0
                    );

                    let load_max = window / 1000 * band.duty_cycle_permille_max;

                    gw::DutyCycleBand {
                        name: band.label.clone(),
                        frequency_min: band.frequency_min,
                        frequency_max: band.frequency_max,
                        load_max: Some(load_max.try_into().unwrap_or_default()),
                        load_tracked: Some((*duration).try_into().unwrap_or_default()),
                    }
                })
                .collect();

            return Some(gw::DutyCycleStats {
                regulation: dc_tracker.get_regulation().into(),
                window: Some(dc_tracker.get_window().try_into().unwrap_or_default()),
                bands: band_stats,
            });
        }

        None
    }

    pub fn enqueue(&mut self, concentrator_count: u32, packet: T) -> Result<(), gw::TxAckStatus> {
        let linear_count = self.get_linear_count(concentrator_count);

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
            TxMode::OnGPS => {
                info!(
                    "Enqueueing packet on pps, downlink_id: {}, counter_us: {}, current_counter_us: {}",
                    packet.get_id(),
                    packet.get_count_us(),
                    concentrator_count,
                );
            }
        }

        if self.full() {
            return Err(gw::TxAckStatus::QueueFull);
        }

        let time_on_air = match packet.get_time_on_air() {
            Ok(v) => v,
            Err(err) => {
                error!("Get time on air for tx packet error, error: {}", err);
                return Err(gw::TxAckStatus::InternalError);
            }
        };

        let mut item = Item {
            // linear_count depends on packet count_us, will be set later
            linear_count: Duration::from_micros(0),
            pre_delay: self.tx_start_delay + self.tx_jit_delay,
            post_delay: time_on_air,
            packet,
        };

        // An immediate downlink becomes a timestamped downlink "ASAP".
        // Set the packet count_us to the first available slot.
        if item.packet.get_tx_mode() == TxMode::Immediate {
            item.packet.set_tx_mode(TxMode::Timestamped);

            // use now + 1 sec
            let mut asap_count = linear_count + Duration::from_secs(1);

            // eventual collision with currently running packet
            // not anymore in queue but still there
            let not_before_count =
                self.tx_linear_count_finished + self.tx_margin_delay + item.pre_delay;
            if asap_count < not_before_count {
                asap_count = not_before_count;
            }

            // check if there is a collision
            if self.collision_test(asap_count, item.pre_delay, item.post_delay) {
                for p in self.items.iter() {
                    asap_count =
                        p.linear_count + p.post_delay + item.pre_delay + self.tx_margin_delay;

                    if !self.collision_test(asap_count, item.pre_delay, item.post_delay) {
                        break;
                    }
                }
            }

            item.linear_count = asap_count;
            item.packet.set_count_us(asap_count.to_concentrator_count());
        } else {
            item.linear_count = self.concentrator_count_to_linear_count(item.packet.get_count_us());
            if (item.packet.get_tx_mode() == TxMode::Timestamped
                || item.packet.get_tx_mode() == TxMode::OnGPS)
                && self.collision_test(item.linear_count, item.pre_delay, item.post_delay)
            {
                return Err(gw::TxAckStatus::CollisionPacket);
            }
        }

        // Is it too late to send this packet?
        if item.linear_count < linear_count
            || item.linear_count - linear_count
                < self.tx_start_delay + self.tx_margin_delay + self.tx_jit_delay
        {
            warn!("Too late to enqueue packet, downlink_id: {}, counter_us: {}, current_counter_us: {}", item.packet.get_id(), item.packet.get_count_us(), concentrator_count);
            return Err(gw::TxAckStatus::TooLate);
        }

        // Is it too early to send this packet?
        if item.linear_count - linear_count > self.tx_max_advance_delay {
            warn!("Too early to enqueue packet, downlink_id: {}, counter_us: {}, current_counter_us: {}", item.packet.get_id(), item.packet.get_count_us(), concentrator_count);
            return Err(gw::TxAckStatus::TooEarly);
        }

        if let Some(dc_tracker) = &mut self.dc_tracker {
            dc_tracker.cleanup(linear_count);

            let res = dc_tracker.try_insert(
                item.packet.get_frequency(),
                item.packet.get_tx_power(),
                dutycycle::Item {
                    start_time: item.linear_count,
                    end_time: item.linear_count + time_on_air,
                },
            );

            if let Err(e) = res {
                match e.downcast_ref::<Error>() {
                    Some(Error::DutyCycle) | Some(Error::DutyCycleFutureItems) => {
                        warn!(
                            "Packet rejected because of duty-cycle, downlink_id: {}",
                            item.packet.get_id()
                        );
                        return Err(gw::TxAckStatus::DutyCycleOverflow);
                    }
                    Some(Error::BandNotFound(f, t)) => {
                        warn!(
                            "No duty-cycle band found for packet, downlink_id: {}, freq: {}, tx_power: {}",
                            item.packet.get_id(), f, t
                        );
                        return Err(gw::TxAckStatus::DutyCycleOverflow);
                    }
                    None => {
                        warn!("Duty-cycle tracker error, error: {}", e);
                        return Err(gw::TxAckStatus::InternalError);
                    }
                }
            }
        }

        debug!(
            "Packet enqueued, downlink_id: {}, count_us: {}",
            item.packet.get_id(),
            item.packet.get_count_us()
        );

        self.items.push(item);
        self.sort();

        Ok(())
    }

    fn get_linear_count(&mut self, concentrator_count: u32) -> Duration {
        // Calculate the diff between the given concentrator_count and the concentrator_count_last,
        // so that we know by how many micro seconds we need to increment the linear_count_last.
        let diff_us = concentrator_count.wrapping_sub(self.concentrator_count_last);
        self.linear_count_last += Duration::from_micros(diff_us as u64);
        self.concentrator_count_last = concentrator_count;
        self.linear_count_last
    }

    fn concentrator_count_to_linear_count(&self, count_us: u32) -> Duration {
        let diff_us = count_us.wrapping_sub(self.concentrator_count_last);
        self.linear_count_last + Duration::from_micros(diff_us as u64)
    }

    fn sort(&mut self) {
        self.items
            .sort_by(|a, b| a.linear_count.cmp(&b.linear_count))
    }

    fn collision_test(&self, count: Duration, pre_delay: Duration, post_delay: Duration) -> bool {
        if count < self.tx_linear_count_finished + pre_delay + self.tx_margin_delay {
            // a packet is currently running, then we need to take it into account
            return true;
        }

        for p2 in self.items.iter() {
            if count > p2.linear_count {
                if count - p2.linear_count <= pre_delay + p2.post_delay + self.tx_margin_delay {
                    return true;
                }
            } else if p2.linear_count - count <= p2.pre_delay + post_delay + self.tx_margin_delay {
                return true;
            }
        }

        false
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
        frequency: u32,
        tx_power: i8,
    }

    impl TxPacket for TxPacketMock {
        fn get_time_on_air(&self) -> Result<Duration> {
            Ok(self.time_on_air)
        }

        fn get_tx_mode(&self) -> TxMode {
            self.tx_mode
        }

        fn get_id(&self) -> u32 {
            0
        }

        fn set_tx_mode(&mut self, tx_mode: TxMode) {
            self.tx_mode = tx_mode;
        }

        fn get_count_us(&self) -> u32 {
            self.count_us
        }

        fn set_count_us(&mut self, count_us: u32) {
            self.count_us = count_us;
        }

        fn get_frequency(&self) -> u32 {
            self.frequency
        }

        fn get_tx_power(&self) -> i8 {
            self.tx_power
        }
    }

    #[test]
    fn test_size() {
        let q: Queue<TxPacketMock> = Queue::new(10, None);
        assert_eq!(10, q.size());
    }

    #[test]
    fn test_enqueue_full() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, None);

        q.enqueue(
            100,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
                frequency: 868100000,
                tx_power: 14,
            },
        )
        .unwrap();

        q.enqueue(
            100,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
                frequency: 868100000,
                tx_power: 14,
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
                    frequency: 868100000,
                    tx_power: 14,
                },
            )
            .is_err(),
            "jit queue should be full"
        );
    }

    #[test]
    fn test_enqueue_immediate() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, None);
        let concentrator_count = 100;

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
                frequency: 868100000,
                tx_power: 14,
            },
        )
        .unwrap();

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
                frequency: 868100000,
                tx_power: 14,
            },
        )
        .unwrap();

        // first item is schedule 1s after concentrator_count.
        let item = &q.items[0];
        assert_eq!(TxMode::Timestamped, item.packet.get_tx_mode());
        assert_eq!(Duration::from_micros(1500 + 40000), item.pre_delay);
        assert_eq!(Duration::from_millis(100), item.post_delay);
        assert_eq!(
            concentrator_count + Duration::from_secs(1).as_micros() as u32,
            item.packet.get_count_us()
        );

        // second item is scheduled after 1st (taking into account the margins).
        let first_end_us = item.packet.get_count_us() + item.post_delay.as_micros() as u32;

        let item = &q.items[1];
        assert_eq!(TxMode::Timestamped, item.packet.get_tx_mode());
        assert_eq!(Duration::from_micros(1500 + 40000), item.pre_delay);
        assert_eq!(Duration::from_millis(100), item.post_delay);
        assert_eq!(
            first_end_us + item.pre_delay.as_micros() as u32 + q.tx_margin_delay.as_micros() as u32,
            item.packet.get_count_us()
        );
    }

    #[test]
    fn test_enqueue_immediate_u32_wrapping() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, None);
        let concentrator_count = 0_u32.wrapping_sub(
            (Duration::from_secs(1)
                + Duration::from_micros(1500 + 40000)
                + Duration::from_millis(100))
            .as_micros() as u32,
        );

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
                frequency: 868100000,
                tx_power: 14,
            },
        )
        .unwrap();

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count_us: 0,
                frequency: 868100000,
                tx_power: 14,
            },
        )
        .unwrap();

        let item = &q.items[0];
        assert_eq!(4294825796, item.packet.get_count_us());

        let item = &q.items[1];
        assert_eq!(1000, item.packet.get_count_us());
    }

    #[test]
    fn test_pop_empty() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, None);

        let item = q.pop(Duration::from_secs(1).as_micros() as u32);
        assert!(item.is_none());
    }

    #[test]
    fn test_pop() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, None);
        let concentrator_count = Duration::from_secs(1).as_micros() as u32;

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Timestamped,
                count_us: Duration::from_secs(2).as_micros() as u32,
                frequency: 868100000,
                tx_power: 14,
            },
        )
        .unwrap();

        let item = q.pop(Duration::from_secs(2).as_micros() as u32);
        assert!(item.is_some());
    }

    #[test]
    fn test_pop_too_far_in_future() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, None);
        let concentrator_count = Duration::from_secs(1).as_micros() as u32;

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Timestamped,
                count_us: Duration::from_secs(2).as_micros() as u32,
                frequency: 868100000,
                tx_power: 14,
            },
        )
        .unwrap();

        let item = q.pop(Duration::from_secs(1).as_micros() as u32);
        assert!(item.is_none());
    }

    #[test]
    fn test_pop_u32_wrapping() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, None);
        let concentrator_count = 0_u32.wrapping_sub(Duration::from_secs(1).as_micros() as u32);

        q.enqueue(
            concentrator_count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Timestamped,
                count_us: 1,
                frequency: 868100000,
                tx_power: 14,
            },
        )
        .unwrap();

        let item = q.pop(0_u32.wrapping_sub(100));
        assert!(item.is_some());
    }
}
