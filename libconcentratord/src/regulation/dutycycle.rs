use std::time::Duration;

use anyhow::Result;

use crate::error::Error;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Item {
    pub start_time: Duration,
    pub end_time: Duration,
}

impl Item {
    /// This returns the duration of the item.
    pub fn duration(&self) -> Duration {
        self.end_time - self.start_time
    }

    /// This returns how much time the item overlaps with the given start and end time.
    /// E.g. this could overlap:
    ///   - zero: if there is no overlap (the item is in the past or future)
    ///   - partial item duration: if there is a partial overlap
    ///   - full item duration: if there is a full overlap
    fn overlapping_duration(&self, start_time: Duration, end_time: Duration) -> Duration {
        if start_time >= self.end_time || end_time <= self.start_time {
            // start_time is after the item ended
            // end time is before the item will start
            Duration::ZERO
        } else if start_time > self.start_time {
            // Start time is after item start time, return partial
            self.end_time - start_time
        } else if end_time < self.end_time {
            // End time is before item end time, return partial
            end_time - self.start_time
        } else {
            self.duration()
        }
    }
}

pub struct Tracker {
    enforce: bool,
    window: Duration,
    max_duration: Duration,
    items: Vec<Item>,
}

impl Tracker {
    pub fn new(window: Duration, max_duration: Duration, enforce: bool) -> Tracker {
        Tracker {
            enforce,
            window,
            max_duration,
            items: Vec::new(),
        }
    }

    /// This cleans up all items of which the start_time and end_time do not overlap
    /// with the given cur_time +/- the configured window for this tracker.
    pub fn cleanup(&mut self, cur_time: Duration) {
        self.items.retain(|i| {
            !i.overlapping_duration(
                cur_time.checked_sub(self.window).unwrap_or(Duration::ZERO),
                cur_time + self.window,
            )
            .is_zero()
        })
    }

    /// This returns the tracked duration at the given time.
    pub fn tracked_duration(&self, cur_time: Duration) -> Duration {
        self.items
            .iter()
            .map(|i| {
                i.overlapping_duration(
                    cur_time.checked_sub(self.window).unwrap_or(Duration::ZERO),
                    cur_time,
                )
            })
            .sum()
    }

    // Try insert the given item. It returns an error in the following case:
    // - If by inserting the item the max_duration would be exceeded
    // - If by inserting the item, it would make already tracked items exceed
    //   the max_duration.
    pub fn try_insert(&mut self, item: Item) -> Result<(), Error> {
        if !self.enforce {
            self.items.push(item);
            return Ok(());
        }

        let tracked = self.tracked_duration(item.end_time);
        if tracked + item.duration() > self.max_duration {
            return Err(Error::DutyCycle);
        }

        for fut_item in self.items.iter().filter(|i| i.start_time > item.start_time) {
            let tracked = self.tracked_duration(fut_item.end_time);
            if tracked + item.duration() > self.max_duration {
                return Err(Error::DutyCycleFutureItems);
            }
        }

        self.items.push(item);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_item_duration() {
        let i = Item {
            start_time: Duration::from_secs(10),
            end_time: Duration::from_secs(11),
        };

        assert_eq!(i.duration(), Duration::from_secs(1))
    }

    #[test]
    fn test_overlapping_duration() {
        struct Test {
            name: String,
            item: Item,
            start_time: Duration,
            end_time: Duration,
            expected: Duration,
        }

        let tests = vec![
            Test {
                name: "full remaining".into(),
                item: Item {
                    start_time: Duration::from_secs(90),
                    end_time: Duration::from_secs(100),
                },
                start_time: Duration::from_secs(90),
                end_time: Duration::from_secs(100),
                expected: Duration::from_secs(10),
            },
            Test {
                name: "full remaining".into(),
                item: Item {
                    start_time: Duration::from_secs(90),
                    end_time: Duration::from_secs(100),
                },
                start_time: Duration::from_secs(80),
                end_time: Duration::from_secs(110),
                expected: Duration::from_secs(10),
            },
            Test {
                name: "take first half".into(),
                item: Item {
                    start_time: Duration::from_secs(90),
                    end_time: Duration::from_secs(100),
                },
                start_time: Duration::from_secs(90),
                end_time: Duration::from_secs(95),
                expected: Duration::from_secs(5),
            },
            Test {
                name: "take second half".into(),
                item: Item {
                    start_time: Duration::from_secs(90),
                    end_time: Duration::from_secs(100),
                },
                start_time: Duration::from_secs(95),
                end_time: Duration::from_secs(105),
                expected: Duration::from_secs(5),
            },
            Test {
                name: "take past".into(),
                item: Item {
                    start_time: Duration::from_secs(90),
                    end_time: Duration::from_secs(100),
                },
                start_time: Duration::from_secs(80),
                end_time: Duration::from_secs(90),
                expected: Duration::from_secs(0),
            },
            Test {
                name: "take future".into(),
                item: Item {
                    start_time: Duration::from_secs(90),
                    end_time: Duration::from_secs(100),
                },
                start_time: Duration::from_secs(100),
                end_time: Duration::from_secs(110),
                expected: Duration::from_secs(0),
            },
        ];

        for tst in &tests {
            assert_eq!(
                tst.expected,
                tst.item.overlapping_duration(tst.start_time, tst.end_time),
                "test: {}",
                tst.name
            );
        }
    }

    #[test]
    fn test_tracker_cleanup() {
        let mut t = Tracker {
            enforce: true,
            window: Duration::from_secs(10),
            max_duration: Duration::ZERO,
            items: vec![
                Item {
                    start_time: Duration::from_secs(0),
                    end_time: Duration::from_secs(1),
                },
                Item {
                    start_time: Duration::from_secs(89),
                    end_time: Duration::from_secs(90),
                },
                Item {
                    start_time: Duration::from_secs(90),
                    end_time: Duration::from_secs(91),
                },
                Item {
                    start_time: Duration::from_secs(109),
                    end_time: Duration::from_secs(110),
                },
                Item {
                    start_time: Duration::from_secs(110),
                    end_time: Duration::from_secs(111),
                },
            ],
        };

        t.cleanup(Duration::from_secs(100));

        assert_eq!(
            t.items,
            vec![
                Item {
                    start_time: Duration::from_secs(90),
                    end_time: Duration::from_secs(91),
                },
                Item {
                    start_time: Duration::from_secs(109),
                    end_time: Duration::from_secs(110),
                },
            ]
        );
    }

    #[test]
    fn test_tracker_cleanup_overlap() {
        let mut t = Tracker {
            enforce: true,
            window: Duration::from_secs(10),
            max_duration: Duration::ZERO,
            items: vec![
                Item {
                    start_time: Duration::from_secs(85),
                    end_time: Duration::from_secs(95),
                },
                Item {
                    start_time: Duration::from_secs(105),
                    end_time: Duration::from_secs(115),
                },
            ],
        };

        t.cleanup(Duration::from_secs(100));

        assert_eq!(
            t.items,
            vec![
                Item {
                    start_time: Duration::from_secs(85),
                    end_time: Duration::from_secs(95),
                },
                Item {
                    start_time: Duration::from_secs(105),
                    end_time: Duration::from_secs(115),
                },
            ]
        );
    }

    #[test]
    fn test_tracker_tracked_duration() {
        struct Test {
            name: String,
            window: Duration,
            items: Vec<Item>,
            time: Duration,
            expected: Duration,
        }

        let tests = vec![
            Test {
                name: "capture full item".into(),
                window: Duration::from_secs(60 * 60),
                items: vec![Item {
                    start_time: Duration::from_secs(3564),
                    end_time: Duration::from_secs(3600),
                }],
                time: Duration::from_secs(3600),
                expected: Duration::from_secs(36),
            },
            Test {
                name: "capture partial item (cutting at the end)".into(),
                window: Duration::from_secs(60 * 60),
                items: vec![Item {
                    start_time: Duration::from_secs(3564),
                    end_time: Duration::from_secs(3600),
                }],
                time: Duration::from_secs(3600 - 18),
                expected: Duration::from_secs(18),
            },
            Test {
                name: "capture partial item (cutting at the beginning)".into(),
                window: Duration::from_secs(60 * 60),
                items: vec![Item {
                    start_time: Duration::from_secs(3564),
                    end_time: Duration::from_secs(3600),
                }],
                time: Duration::from_secs(3582 + 3600),
                expected: Duration::from_secs(18),
            },
        ];

        for tst in &tests {
            let t = Tracker {
                enforce: true,
                window: tst.window,
                max_duration: Duration::ZERO,
                items: tst.items.clone(),
            };

            assert_eq!(
                tst.expected,
                t.tracked_duration(tst.time),
                "test: {}",
                tst.name
            );
        }
    }

    #[test]
    fn test_tracker_try_insert() {
        struct Test {
            name: String,
            window: Duration,
            max_duration: Duration,
            items: Vec<Item>,
            insert_item: Item,
            can_insert: bool,
        }

        let tests = vec![
            // The tracker is empty and the inserted item has exactly the
            // max tracker duration.
            Test {
                name: "empty tracker - item fits".into(),
                window: Duration::from_secs(3600),
                max_duration: Duration::from_secs(36),
                items: vec![],
                insert_item: Item {
                    start_time: Duration::from_secs(0),
                    end_time: Duration::from_secs(36),
                },
                can_insert: true,
            },
            // The tracker is empty, but the item does not fit as the duration
            // exceeds the max tracker duration.
            Test {
                name: "empty tracker - item exceeds max duration".into(),
                window: Duration::from_secs(3600),
                max_duration: Duration::from_secs(36),
                items: vec![],
                insert_item: Item {
                    start_time: Duration::from_secs(0),
                    end_time: Duration::from_secs(37),
                },
                can_insert: false,
            },
            // There are two items tracked (24s), the inserted item still fits.
            Test {
                name: "item fits".into(),
                window: Duration::from_secs(3600),
                max_duration: Duration::from_secs(36),
                items: vec![
                    Item {
                        start_time: Duration::from_secs(0),
                        end_time: Duration::from_secs(12),
                    },
                    Item {
                        start_time: Duration::from_secs(20),
                        end_time: Duration::from_secs(32),
                    },
                ],
                insert_item: Item {
                    start_time: Duration::from_secs(40),
                    end_time: Duration::from_secs(52),
                },
                can_insert: true,
            },
            // There are three items (36s) and within the same window the item
            // will thus not fit.
            Test {
                name: "item does not fit".into(),
                window: Duration::from_secs(3600),
                max_duration: Duration::from_secs(36),
                items: vec![
                    Item {
                        start_time: Duration::from_secs(0),
                        end_time: Duration::from_secs(12),
                    },
                    Item {
                        start_time: Duration::from_secs(20),
                        end_time: Duration::from_secs(32),
                    },
                    Item {
                        start_time: Duration::from_secs(40),
                        end_time: Duration::from_secs(52),
                    },
                ],
                insert_item: Item {
                    start_time: Duration::from_secs(60),
                    end_time: Duration::from_secs(61),
                },
                can_insert: false,
            },
            // During the start/end time of the inserted item, the first
            // existing item slides out of the tracked window thus it can
            // be inserted as it "frees up 12sec".
            Test {
                name: "1st item slides out of window - item fits".into(),
                window: Duration::from_secs(3600),
                max_duration: Duration::from_secs(36),
                items: vec![
                    Item {
                        start_time: Duration::from_secs(0),
                        end_time: Duration::from_secs(12),
                    },
                    Item {
                        start_time: Duration::from_secs(20),
                        end_time: Duration::from_secs(32),
                    },
                    Item {
                        start_time: Duration::from_secs(40),
                        end_time: Duration::from_secs(52),
                    },
                ],
                insert_item: Item {
                    start_time: Duration::from_secs(3600),
                    end_time: Duration::from_secs(3612),
                },
                can_insert: true,
            },
            // Same as the previous test, but while 12sec is beeing "freed up",
            // we try to insert an item with a duration of 13sec which exceeds
            // the available max duration.
            Test {
                name: "1st item slides out of window - item does not fit".into(),
                window: Duration::from_secs(3600),
                max_duration: Duration::from_secs(36),
                items: vec![
                    Item {
                        start_time: Duration::from_secs(0),
                        end_time: Duration::from_secs(12),
                    },
                    Item {
                        start_time: Duration::from_secs(20),
                        end_time: Duration::from_secs(32),
                    },
                    Item {
                        start_time: Duration::from_secs(40),
                        end_time: Duration::from_secs(52),
                    },
                ],
                insert_item: Item {
                    start_time: Duration::from_secs(3600),
                    end_time: Duration::from_secs(3613),
                },
                can_insert: false,
            },
            // At the moment of insert, the first item slides out of the window,
            // however, it would make the item at 3612 exceed the max duration.
            Test {
                name: "1st item slides out of window - item does not fit because of future item"
                    .into(),
                window: Duration::from_secs(3600),
                max_duration: Duration::from_secs(36),
                items: vec![
                    Item {
                        start_time: Duration::from_secs(0),
                        end_time: Duration::from_secs(12),
                    },
                    Item {
                        start_time: Duration::from_secs(20),
                        end_time: Duration::from_secs(32),
                    },
                    Item {
                        start_time: Duration::from_secs(40),
                        end_time: Duration::from_secs(52),
                    },
                    Item {
                        start_time: Duration::from_secs(3612),
                        end_time: Duration::from_secs(3624),
                    },
                ],
                insert_item: Item {
                    start_time: Duration::from_secs(3600),
                    end_time: Duration::from_secs(3612),
                },
                can_insert: false,
            },
            // At the moment of insert the first item slides out of the window.
            // The future item scheduled at 3620 can still be transmitted as
            // during that time, the second item (20) slides out of the window.
            Test {
                name: "1st two item slides out of window - item does fit".into(),
                window: Duration::from_secs(3600),
                max_duration: Duration::from_secs(36),
                items: vec![
                    Item {
                        start_time: Duration::from_secs(0),
                        end_time: Duration::from_secs(12),
                    },
                    Item {
                        start_time: Duration::from_secs(20),
                        end_time: Duration::from_secs(32),
                    },
                    Item {
                        start_time: Duration::from_secs(40),
                        end_time: Duration::from_secs(52),
                    },
                    Item {
                        start_time: Duration::from_secs(3620),
                        end_time: Duration::from_secs(3632),
                    },
                ],
                insert_item: Item {
                    start_time: Duration::from_secs(3600),
                    end_time: Duration::from_secs(3612),
                },
                can_insert: true,
            },
        ];

        for tst in &tests {
            let mut t = Tracker {
                enforce: true,
                window: tst.window,
                max_duration: tst.max_duration,
                items: tst.items.clone(),
            };

            assert_eq!(
                tst.can_insert,
                t.try_insert(tst.insert_item.clone()).is_ok(),
                "test: {}",
                tst.name
            );

            if tst.can_insert {
                assert_eq!(&tst.insert_item, t.items.last().unwrap());
            }
        }
    }
}
