mod item;
mod tracker;

use std::fmt;
use std::time::Duration;

use log::{debug, info, warn};

use crate::jitqueue;

use super::standard;
use item::Item;
use tracker::Tracker;

#[derive(Debug)]
pub enum Error {
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
                                    warn!("Band {}, max tx time reached, current effective load: {:.3}%, estimated overlow: {:.2}/{:.2}% ({:.1}%)",
                                        tracker.get_band(),
                                        tracker.simulated_load * 100.0,
                                        overload * 100.0,
                                        max * 100.0,
                                        (overload / max) * 100.0);
                                    //TODO: Should return a specific error like TxAckStatus::DutyCycleOverlow
                                    return Err(chirpstack_api::gw::TxAckStatus::InternalError);
                                }
                            }
                        }
                        Ok(simulated_load) => {
                            // simulation succeed, push new item to common queue
                            self.items.push(item);
                            self.items.sort();

                            info!(
                                "Band {}, push new item, airtime: {:?}, new effective load: {:.2}/{:.2}% ({:.1}%)",
                                tracker.get_band(),
                                item.airtime,
                                simulated_load * 100.0,
                                tracker.load_max * 100.0,
                                (simulated_load / tracker.load_max) * 100.0
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
                item.tx_power, tracker_matches,
            );
            return Err(chirpstack_api::gw::TxAckStatus::TxPower);
        } else {
            warn!(
                "No tracker found for packet, tx power: {}, frequency: {}",
                item.tx_power, item.frequency,
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
                line.push_str("Duty cycle stats (effective loads):\n");
                for (i, tracker) in self.trackers.iter().enumerate() {
                    line.push_str(&format!(
                        "  [{}]: {:.2}/{:.2}% ({:.1}%)",
                        tracker.get_band(),
                        tracker.simulated_load * 100.0,
                        tracker.load_max * 100.0,
                        (tracker.simulated_load / tracker.load_max) * 100.0
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

// Used only for debug but it still usefull with lînear timing
// Should be kept ? moved ?
pub struct DurationDisplay(Duration);
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
