use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;

use log::{debug, error, info, trace, warn};

use super::super::jitqueue;
use super::standard;

#[derive(Debug)]
enum Error {
    DutyCycleOverflow(f32, f32),
    TrackerMismatch,
}
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::DutyCycleOverflow(overload, max) => {
                write!(
                    f,
                    "Duty cycle overload: {:.2}/{:.2}% ({:.1}%)",
                    overload * 100.0,
                    max * 100.0,
                    (overload / max) * 100.0
                )
            }
            Error::TrackerMismatch => {
                write!(f, "Tracker doesn't match specified item")
            }
        }
    }
}

#[derive(Copy, Clone)]
struct Item<T> {
    packet: T,
    time: Duration,
    airtime: Duration,
    aggregated: bool,
}

struct Tracker<T> {
    max_load: f32,
    band: standard::Band,
    residual_load: f32,
    residual_load_time: Duration,
    coef_load_tx: f32,
    coef_load_idle: f32,
    time_window_us: u32,
    time_last: Duration,
    cleanup_time: Duration,
    phantom: PhantomData<T>,
}

pub struct Engine<T> {
    standard: standard::Standard,
    config: standard::Configuration,
    items: Vec<Item<T>>,
    trackers: Vec<Tracker<T>>,
    display_time: Duration,
}

impl<T: jitqueue::TxPacket + Copy> Engine<T> {
    pub fn new(capacity: usize, standard: standard::Standard) -> Self {
        let config = standard.get_configuration();
        let mut engine = Engine {
            standard,
            config,
            items: Vec::with_capacity(capacity),
            trackers: vec![],
            display_time: Default::default(),
        };
        // probably not the best way to do it but at least, it keeps
        // this responsability on regul. engine (and not jitqueue)
        if let standard::Standard::None = standard {
            info!("Regulation standard is not defined, engine disabled");
            return engine;
        }
        for band in engine.config.bands.iter() {
            engine
                .trackers
                .push(Tracker::new(&band, engine.config.window_time));
        }
        info!(
            "Initializing regulation engine, standard: {:?}, bands: {}",
            standard,
            engine.trackers.len()
        );
        engine
    }

    pub fn enqueue(
        &mut self,
        time_last: Duration,
        jititem: &jitqueue::Item<T>,
    ) -> Result<(), chirpstack_api::gw::TxAckStatus> {
        if let standard::Standard::None = self.standard {
            return Ok(());
        }

        self.update_time(time_last);

        // derive item from jititem
        let item = Item::new(jititem);
        let mut tracker_matches = 0;
        for tracker in self.trackers.iter_mut() {
            if tracker.matching_frequency(&item) {
                tracker_matches = tracker_matches + 1;

                if tracker.matching_tx_power(&item) {
                    // tracker match frequency and power, simulate new load
                    match tracker.simulate_load(&mut self.items, &item) {
                        Err(e) => {
                            match e {
                                Error::TrackerMismatch => {}
                                Error::DutyCycleOverflow(overload, max) => {
                                    warn!("Band {}, max tx time reached, current load: {:.3}%, estimated overlow: {:.2}/{:.2}% ({:.1}%)",
                                        tracker.band,
                                        tracker.residual_load * 100.0,
                                        overload * 100.0,
                                        max * 100.0,
                                        (overload / max) * 100.0);
                                    //TODO: Should return a specific error like TxAckStatus::DutyCycleOverlow
                                    return Err(chirpstack_api::gw::TxAckStatus::InternalError);
                                }
                            }
                        }
                        Ok(load) => {
                            // simulation succeed, push new item to common queue
                            self.items.push(item);
                            self.items.sort();

                            info!(
                                "Band {}, push new item, airtime: {:?}, new load: {:.2}/{:.2}% ({:.1}%)",
                                tracker.band,
                                item.airtime,
                                load * 100.0,
                                tracker.max_load * 100.0,
                                (load / tracker.max_load) * 100.0
                            );

                            return Ok(());
                        }
                    }
                }
            }
        }

        // that's still ugly but don't forget that some standard (like EN 300 220)
        // have some band sharing the same frequencies but with different max power
        // and duty cycle (band P for exemple).
        // This is assumed here we can TX on both band as long as power and duty
        // cycle are respected for each but not really sure about that, are they
        // exclusive ?
        if tracker_matches > 0 {
            // so we have found at least one band which support this frequencies
            // but power was too high.
            warn!(
                "Packet TX power is too high, rejected, tx power: {}, matching tracker: {}",
                item.packet.get_tx_power(),
                tracker_matches,
            );
            return Err(chirpstack_api::gw::TxAckStatus::TxPower);
        } else {
            warn!(
                "No tracker found for packet, tx power: {}, frequency: {}",
                item.packet.get_tx_power(),
                item.packet.get_frequency(),
            );
            return Err(chirpstack_api::gw::TxAckStatus::InternalError);
        }
    }

    pub fn cleanup(&mut self, time_last: Duration) {
        if let standard::Standard::None = self.standard {
            return;
        }

        self.update_time(time_last);
        for tracker in self.trackers.iter_mut() {
            tracker.cleanup(&mut self.items);
        }
    }

    fn update_time(&mut self, time: Duration) {
        // Currently fixed to 30 seconds but should probably follow
        // a config parameters like stats_interval.
        if time - self.display_time > Duration::from_secs(30) {
            debug!("{}", {
                let mut line = String::new();
                line.push_str("Duty cycle stats:\n");
                for (i, tracker) in self.trackers.iter().enumerate() {
                    line.push_str(&format!(
                        "  [{}]: {:.2}/{:.2}% ({:.1}%)",
                        tracker.band,
                        tracker.residual_load * 100.0,
                        tracker.max_load * 100.0,
                        (tracker.residual_load / tracker.max_load) * 100.0
                    ));
                    if i < self.trackers.len() - 1 {
                        line.push_str("\n");
                    }
                }
                line
            });
            self.display_time = time;
        }

        for tracker in self.trackers.iter_mut() {
            tracker.update_time(time);
        }
    }
}

impl<T: jitqueue::TxPacket + Copy> Tracker<T> {
    fn new(band: &standard::Band, time_window: Duration) -> Self {
        let max_load = band.duty_cycle_percent_max / 100.0;
        Tracker {
            max_load: max_load,
            band: band.clone(),
            residual_load: 0.0,
            residual_load_time: Duration::from_millis(0),
            coef_load_tx: 1.0,
            coef_load_idle: -max_load,
            time_window_us: time_window.as_micros() as u32,
            time_last: Duration::from_micros(0),
            cleanup_time: Duration::from_micros(0),
            phantom: PhantomData,
        }
    }

    pub fn update_time(&mut self, time: Duration) {
        self.time_last = time;
    }

    pub fn matching(&self, item: &Item<T>) -> bool {
        self.matching_frequency(item) && self.matching_tx_power(&item)
    }

    pub fn matching_frequency(&self, item: &Item<T>) -> bool {
        let frequency = item.packet.get_frequency();
        frequency >= self.band.frequency_min && frequency < self.band.frequency_max
    }

    pub fn matching_tx_power(&self, item: &Item<T>) -> bool {
        item.packet.get_tx_power() <= self.band.tx_power_max
    }

    pub fn simulate_load(
        &mut self,
        mut base_items: &mut Vec<Item<T>>,
        item: &Item<T>,
    ) -> Result<f32, Error> {
        if !self.matching(item) {
            // this packet doesn't concern this tracker
            error!(
                "pkt doesn't match tracker, frequency: {}, power: {}",
                item.packet.get_frequency(),
                item.packet.get_tx_power()
            );
            return Err(Error::TrackerMismatch);
        }

        if let Err(e) = self.aggregate_past(&mut base_items) {
            error!("Update in returned error:\n{}", e);
        }

        self.remaining_test(&base_items, item)
    }

    pub fn cleanup(&mut self, mut engine_items: &mut Vec<Item<T>>) {
        if self.time_last - self.cleanup_time > Duration::from_secs(1) {
            if let Err(e) = self.aggregate_past(&mut engine_items) {
                error!("Update out returned error:\n{}", e);
            }
        }
    }

    fn remaining_test(&self, base_items: &Vec<Item<T>>, item: &Item<T>) -> Result<f32, Error> {
        // create a vector of item concerned by this tracker
        let mut items = self.filter(base_items);
        // push the new item, no collision assumed as already treated by the jitqueue
        items.push(item.clone());
        items.sort();

        // find farthest item time in future
        let mut end_time = self.time_last;
        if let Some(last_future_item) = items.last() {
            let end_item_time = last_future_item.time + last_future_item.airtime;
            if end_item_time > end_time {
                end_time = end_item_time;
            }
        }
        self.load_calc_over_time(
            self.residual_load,
            self.residual_load_time,
            end_time,
            &items,
        )
    }

    fn load_calc_tx(&self, mut load_in: f32, item: &Item<T>) -> Result<f32, Error> {
        if item.airtime > Duration::from_micros(0) {
            let time_ratio = item.airtime.as_micros() as f32 / self.time_window_us as f32;
            load_in = load_in + time_ratio * (self.coef_load_tx + self.coef_load_idle);
        }
        if load_in > self.max_load {
            return Err(Error::DutyCycleOverflow(load_in, self.max_load));
        }
        Ok(load_in)
    }

    fn load_calc_idle(
        &self,
        mut load_in: f32,
        start_time: Duration,
        end_time: Duration,
    ) -> Result<f32, Error> {
        if end_time == start_time {
            return Ok(load_in);
        }
        if end_time > start_time {
            let diff_time = end_time - start_time;
            let time_ratio = diff_time.as_micros() as f32 / self.time_window_us as f32;
            load_in = load_in + time_ratio * self.coef_load_idle;
            if load_in < 0.0 {
                load_in = 0.0;
            }
        } else {
            warn!("Load calculation, end_time is before or equal start time. current time: {}, start_time: {}, end_time: {}",
                DurationDisplay(self.time_last),
                DurationDisplay(start_time),
                DurationDisplay(end_time)
            );
        }
        return Ok(load_in);
    }

    // returns min/max load over the specified period
    // start_time must not be during TX packet
    // end_time must not be during TX packet
    fn load_calc_over_time(
        &self,
        mut load_in: f32,
        start_time: Duration,
        end_time: Duration,
        items: &Vec<Item<T>>,
    ) -> Result<f32, Error> {
        // memorize deviation from idle load (0.0%)
        let mut time_cursor = start_time;

        // find items in range
        let count = items
            .iter()
            .filter(|&item| {
                return item.time + item.airtime <= end_time;
            })
            .count();

        trace!(
            "load calc, load in: {}, start time: {}, end_time: {}, diff time: {}, item count: {}, current time: {}",
            load_in,
            DurationDisplay(start_time),
            DurationDisplay(end_time),
            DurationDisplay(end_time - start_time),
            count,
            DurationDisplay(self.time_last),
        );

        if count == 0 {
            // special case, there is no TX packet inside the specified range
            load_in = self.load_calc_idle(load_in, start_time, end_time)?;
        } else {
            // first calculate load between start_time and first pkt
            if let Some(item) = items.first() {
                load_in = self.load_calc_idle(load_in, time_cursor, item.time)?;
                time_cursor = item.time;
            }
            // then calculate TX load and idle for other packets
            for (pos, item) in items.iter().enumerate() {
                let end_time = item.time + item.airtime;
                load_in = self.load_calc_tx(load_in, &item)?;
                time_cursor = end_time;

                // is not the last one ?
                if pos != count - 1 {
                    let end_idle_time = items[pos + 1].time;
                    load_in = self.load_calc_idle(load_in, time_cursor, end_idle_time)?;
                    time_cursor = end_idle_time;
                }
            }
            // then we add remaining idle time
            if time_cursor < end_time {
                load_in = self.load_calc_idle(load_in, time_cursor, end_time)?;
            }
        }
        Ok(load_in)
    }

    fn aggregate_past(&mut self, engine_items: &mut Vec<Item<T>>) -> Result<(), Error> {
        let mut past_items: Vec<Item<T>> = vec![];
        let mut end_time = self.time_last;

        // find past items which match this tracker
        for item in engine_items.iter_mut() {
            if !self.matching(&item) {
                // this item is not concerned by this tracker
                continue;
            }
            if item.time + item.airtime > end_time {
                if item.time < end_time {
                    end_time = item.time;
                }
                break;
            }
            // There is probably a better way than cloning item, need opti.
            past_items.push(item.clone());
            item.aggregated = true;
        }

        // remove past item from engine queue
        engine_items.retain(|item| !item.aggregated);

        // update last load based on past item
        trace!(
            "cleanup past, residual_load: {}, residual_load_time: {}, end_time: {}, diff time: {}, past items: {}, remain items: {}, current time: {}",
            self.residual_load,
            DurationDisplay(self.residual_load_time),
            DurationDisplay(end_time),
            DurationDisplay(end_time - self.residual_load_time),
            past_items.len(),
            engine_items.len(),
            DurationDisplay(self.time_last),
        );

        self.residual_load = self.load_calc_over_time(
            self.residual_load,
            self.residual_load_time,
            end_time,
            &past_items,
        )?;
        self.residual_load_time = end_time;
        self.cleanup_time = self.time_last;
        Ok(())
    }

    fn filter(&self, items: &Vec<Item<T>>) -> Vec<Item<T>> {
        return items
            .iter()
            .filter(|&item| return self.matching(item))
            .cloned()
            .collect();
    }
}

impl<T: jitqueue::TxPacket + Copy> Item<T> {
    pub fn new(item: &jitqueue::Item<T>) -> Item<T> {
        Item {
            packet: item.packet,
            time: item.time,
            airtime: item.post_delay,
            aggregated: false,
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

// Used only for debug but it still usefull with lînear timing
// Should be kept ? moved ?
struct DurationDisplay(Duration);
impl fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut seconds = self.0.as_secs();
        let millis = self.0.subsec_millis();

        let mut minutes = seconds / 60;
        seconds = seconds % 60;

        let hours = minutes / 60;
        minutes = minutes % 60;

        write!(
            f,
            "{:02}:{:02}:{:02}.{:03}",
            hours, minutes, seconds, millis
        )
    }
}
