use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use log::info;

use super::dutycycle;
use super::standard;
use crate::helpers::ToConcentratorCount;

pub struct Tracker {
    config: standard::Configuration,
    trackers: HashMap<standard::Band, dutycycle::Tracker>,
}

impl Tracker {
    pub fn new(config: standard::Configuration) -> Self {
        Tracker {
            config,
            trackers: HashMap::new(),
        }
    }

    pub fn try_insert(&mut self, tx_freq: u32, tx_power: i8, item: dutycycle::Item) -> Result<()> {
        let band = self.config.get_band(tx_freq, tx_power)?;

        match self.trackers.get_mut(&band) {
            Some(tracker) => tracker.try_insert(item.clone())?,
            None => {
                let mut tracker = dutycycle::Tracker::new(
                    self.config.window_time,
                    self.config.window_time / 1000 * band.duty_cycle_permille_max,
                );
                tracker.try_insert(item.clone())?;
            }
        };

        info!("Item tracked, band: {}, freq: {}, tx_power: {}, start_counter_us: {}, end_counter_us: {}, duration: {:?}", band.label, tx_freq, tx_power, item.start_time.to_concentrator_count(), item.end_time.to_concentrator_count(), item.duration());

        Ok(())
    }

    pub fn cleanup(&mut self, cur_time: Duration) {
        for v in self.trackers.values_mut() {
            v.cleanup(cur_time);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Item {
        tx_freq: u32,
        tx_power: i8,
        start_time: Duration,
        end_time: Duration,
    }

    struct Test {
        name: String,
        items: Vec<Item>,
        ok: bool,
    }

    #[test]
    fn test_etsi_en_300_220() {
        let tests = vec![
            Test {
                name: "K - 0.1%".into(),
                items: vec![Item {
                    tx_freq: 863000000,
                    tx_power: 14,
                    start_time: Duration::from_millis(0),
                    end_time: Duration::from_millis(3600),
                }],
                ok: true,
            },
            Test {
                name: "K - > 0.1%".into(),
                items: vec![Item {
                    tx_freq: 863000000,
                    tx_power: 14,
                    start_time: Duration::from_millis(0),
                    end_time: Duration::from_millis(3601),
                }],
                ok: false,
            },
            Test {
                name: "M - 1%".into(),
                items: vec![Item {
                    tx_freq: 868000000,
                    tx_power: 14,
                    start_time: Duration::from_millis(0),
                    end_time: Duration::from_millis(36000),
                }],
                ok: true,
            },
            Test {
                name: "M - >1%".into(),
                items: vec![Item {
                    tx_freq: 868000000,
                    tx_power: 14,
                    start_time: Duration::from_millis(0),
                    end_time: Duration::from_millis(36001),
                }],
                ok: false,
            },
            Test {
                name: "L - 1% - M 1%".into(),
                items: vec![
                    Item {
                        tx_freq: 865000000,
                        tx_power: 14,
                        start_time: Duration::from_millis(0),
                        end_time: Duration::from_millis(36000),
                    },
                    Item {
                        tx_freq: 868000000,
                        tx_power: 14,
                        start_time: Duration::from_millis(36000),
                        end_time: Duration::from_millis(72000),
                    },
                ],
                ok: true,
            },
            Test {
                name: "Invalid freq".into(),
                items: vec![Item {
                    tx_freq: 920000000,
                    tx_power: 14,
                    start_time: Duration::from_millis(0),
                    end_time: Duration::from_millis(1),
                }],
                ok: false,
            },
            Test {
                name: "K - invalid tx_power".into(),
                items: vec![Item {
                    tx_freq: 863000000,
                    tx_power: 15,
                    start_time: Duration::from_millis(0),
                    end_time: Duration::from_millis(3600),
                }],
                ok: false,
            },
            Test {
                name: "K - lower tx_power (valid)".into(),
                items: vec![Item {
                    tx_freq: 863000000,
                    tx_power: 10,
                    start_time: Duration::from_millis(0),
                    end_time: Duration::from_millis(3600),
                }],
                ok: true,
            },
        ];

        for tst in &tests {
            let conf = standard::Configuration::new(standard::Standard::ETSI_EN_300_220);
            let mut tracker = Tracker::new(conf);
            for item in &tst.items {
                assert_eq!(
                    tst.ok,
                    tracker
                        .try_insert(
                            item.tx_freq,
                            item.tx_power,
                            dutycycle::Item {
                                start_time: item.start_time,
                                end_time: item.end_time
                            }
                        )
                        .is_ok(),
                    "{}",
                    tst.name,
                );
            }
        }
    }
}
