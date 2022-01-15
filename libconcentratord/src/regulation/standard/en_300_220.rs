use std::time::Duration;

use super::{Band, Configuration};

pub fn get_configuration() -> Configuration {
    return Configuration {
        bands: vec![
            Band {
                label: "K".to_owned(),
                frequency_min: 863000000,
                frequency_max: 865000000,
                duty_cycle_percent_max: 0.1,
                tx_power_max: 14,
            },
            Band {
                label: "L".to_owned(),
                frequency_min: 865000000,
                frequency_max: 868000000,
                duty_cycle_percent_max: 1.0,
                tx_power_max: 14,
            },
            Band {
                label: "M".to_owned(),
                frequency_min: 868000000,
                frequency_max: 868600000,
                duty_cycle_percent_max: 1.0,
                tx_power_max: 14,
            },
            Band {
                label: "N".to_owned(),
                frequency_min: 868700000,
                frequency_max: 869200000,
                duty_cycle_percent_max: 0.1,
                tx_power_max: 14,
            },
            Band {
                label: "P".to_owned(),
                frequency_min: 869400000,
                frequency_max: 869650000,
                duty_cycle_percent_max: 10.0,
                tx_power_max: 27,
            },
            Band {
                label: "P".to_owned(),
                frequency_min: 869700000,
                frequency_max: 870000000,
                duty_cycle_percent_max: 100.0,
                tx_power_max: 7,
            },
            Band {
                label: "Q".to_owned(),
                frequency_min: 869700000,
                frequency_max: 870000000,
                duty_cycle_percent_max: 1.0,
                tx_power_max: 14,
            },
        ],
        window_time: Duration::from_secs(60 * 60),
    };
}
