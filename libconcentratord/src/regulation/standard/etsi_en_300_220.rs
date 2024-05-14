use std::time::Duration;

use super::{Band, Configuration};

pub fn new() -> Configuration {
    Configuration {
        regulation: chirpstack_api::common::Regulation::EtsiEn300220,
        bands: vec![
            Band {
                label: "K".into(),
                frequency_min: 863000000,
                frequency_max: 865000000,
                duty_cycle_permille_max: 1,
                tx_power_max_eirp: 14 + 2,
            },
            Band {
                label: "L".into(),
                frequency_min: 865000000,
                frequency_max: 868000000,
                duty_cycle_permille_max: 10,
                tx_power_max_eirp: 14 + 2,
            },
            Band {
                label: "M".into(),
                frequency_min: 868000000,
                frequency_max: 868600000,
                duty_cycle_permille_max: 10,
                tx_power_max_eirp: 14 + 2,
            },
            Band {
                label: "N".into(),
                frequency_min: 868700000,
                frequency_max: 869200000,
                duty_cycle_permille_max: 1,
                tx_power_max_eirp: 14 + 2,
            },
            Band {
                label: "P".into(),
                frequency_min: 869400000,
                frequency_max: 869650000,
                duty_cycle_permille_max: 100,
                tx_power_max_eirp: 27 + 2,
            },
            Band {
                label: "P".into(),
                frequency_min: 869700000,
                frequency_max: 870000000,
                duty_cycle_permille_max: 1000,
                tx_power_max_eirp: 7 + 2,
            },
            Band {
                label: "Q".into(),
                frequency_min: 869700000,
                frequency_max: 870000000,
                duty_cycle_permille_max: 10,
                tx_power_max_eirp: 14 + 2,
            },
        ],
        window_time: Duration::from_secs(60 * 60),
    }
}
