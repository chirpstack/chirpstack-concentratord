use std::time::Duration;

use log::{error, trace};

use super::standard;
use super::{DurationDisplay, Error, Item};

pub struct Tracker {
    pub load_max: f32,
    band: standard::Band,
    coef_load_tx: f32,
    coef_load_idle: f32,
    time_window_us: u32,

    pub residual_load: f32,
    residual_load_time: Duration,
    pub simulated_load: f32,
    time_last: Duration,
}

impl Tracker {
    pub fn new(band: &standard::Band, time_window: Duration) -> Self {
        let load_max = band.duty_cycle_percent_max / 100.0;
        Tracker {
            load_max,
            band: band.clone(),
            coef_load_tx: 1.0,
            coef_load_idle: -load_max,
            time_window_us: time_window.as_micros() as u32,

            residual_load: 0.0,
            residual_load_time: Duration::from_millis(0),
            simulated_load: 0.0,
            time_last: Duration::from_micros(0),
        }
    }

    pub fn get_band(&self) -> &standard::Band {
        &self.band
    }

    pub fn update_time(&mut self, time: Duration) {
        self.time_last = time;
    }

    pub fn matching(&self, item: &Item) -> bool {
        self.matching_frequency(item) && self.matching_tx_power(&item)
    }

    pub fn matching_frequency(&self, item: &Item) -> bool {
        let frequency = item.frequency;
        frequency >= self.band.frequency_min && frequency < self.band.frequency_max
    }

    pub fn matching_tx_power(&self, item: &Item) -> bool {
        item.tx_power <= self.band.tx_power_max
    }

    pub fn simulate_load(
        &mut self,
        mut engine_items: &mut Vec<Item>,
        item: &Item,
    ) -> Result<f32, Error> {
        if !self.matching(item) {
            // this packet doesn't concern this tracker
            error!(
                "Packet doesn't match tracker, frequency: {}, power: {}",
                item.frequency, item.tx_power,
            );
            return Err(Error::TrackerMismatch);
        }

        if let Err(e) = self.aggregate_past(&mut engine_items) {
            error!("Load simulation returned error:\n{}", e);
        }

        self.simulated_load = self.remaining_test(&engine_items, Some(item))?;
        Ok(self.simulated_load)
    }

    pub fn cleanup(&mut self, mut engine_items: &mut Vec<Item>) {
        if let Err(e) = self.aggregate_past(&mut engine_items) {
            error!("Tracker cleanup returned error:\n{}", e);
        }
        self.simulated_load = self.remaining_test(&engine_items, None).unwrap();
    }

    fn remaining_test(
        &self,
        engine_items: &Vec<Item>,
        new_item: Option<&Item>,
    ) -> Result<f32, Error> {
        // create a vector of item concerned by this tracker
        let mut items = self.filter(engine_items);
        // push the new item, no collision assumed as already treated by the jitqueue
        if let Some(item) = new_item {
            items.push(item.clone());
            items.sort();
        }

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

    fn load_calc_tx(&self, mut load_in: f32, item: &Item) -> Result<f32, Error> {
        if item.airtime > Duration::from_micros(0) {
            let time_ratio = item.airtime.as_micros() as f32 / self.time_window_us as f32;
            load_in = load_in + time_ratio * (self.coef_load_tx + self.coef_load_idle);
        }
        if load_in > self.load_max {
            return Err(Error::DutyCycleOverflow(load_in, self.load_max));
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
            error!("Load calculation, end_time is before start_time. current time: {}, start_time: {}, end_time: {}",
                DurationDisplay(self.time_last),
                DurationDisplay(start_time),
                DurationDisplay(end_time)
            );
        }
        return Ok(load_in);
    }

    // returns new load based on input load with check for overload and neg. limit to 0
    // start_time must not be during TX packet
    // end_time must not be during TX packet
    fn load_calc_over_time(
        &self,
        mut load_in: f32,
        start_time: Duration,
        end_time: Duration,
        items: &Vec<Item>,
    ) -> Result<f32, Error> {
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

    fn aggregate_past(&mut self, engine_items: &mut Vec<Item>) -> Result<(), Error> {
        let mut past_items: Vec<Item> = vec![];
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

        // update residual load based on past item
        trace!(
            "Past items aggregation, residual_load: {}, residual_load_time: {}, end_time: {}, diff time: {}, past items: {}, remain items: {}, current time: {}",
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
        Ok(())
    }

    fn filter(&self, items: &Vec<Item>) -> Vec<Item> {
        return items
            .iter()
            .filter(|&item| return self.matching(item))
            .cloned()
            .collect();
    }
}
