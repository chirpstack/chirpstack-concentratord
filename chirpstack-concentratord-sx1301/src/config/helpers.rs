const DEFAULT_RADIO_BANDWIDTH: u32 = 925000;

struct ChannelConfiguration {
    pub freq_hz: u32,
    pub bandwidth: u32,
}

impl ChannelConfiguration {
    fn min_radio_freq(&self) -> u32 {
        let radio_bw = get_radio_bandwidth(self.bandwidth);
        return self.freq_hz - (self.bandwidth / 2) + (radio_bw / 2);
    }
}

fn get_radio_bandwidth(channel_bw: u32) -> u32 {
    match channel_bw {
        500000 => 1100000,
        250000 => 1000000,
        125000 => 925000,
        _ => DEFAULT_RADIO_BANDWIDTH,
    }
}

pub fn get_radio_frequencies(config: &super::Configuration) -> Result<Vec<u32>, String> {
    let mut channels: Vec<ChannelConfiguration> = Vec::new();
    let mut radios = vec![0; config.gateway.model_config.radio_count];

    // add channels to vector
    for c in config.gateway.concentrator.multi_sf_channels.iter() {
        if *c != 0 {
            channels.push(ChannelConfiguration {
                freq_hz: *c,
                bandwidth: 125000,
            });
        }
    }

    // sort vector by min radio freq
    channels.sort_by(|a, b| a.min_radio_freq().cmp(&b.min_radio_freq()));

    for c in channels {
        let channel_max = c.freq_hz - (c.bandwidth / 2);
        let radio_bw = get_radio_bandwidth(c.bandwidth);
        let min_radio_center_freq = c.freq_hz - (c.bandwidth / 2) + (radio_bw / 2);

        for (i, radio_freq) in radios.iter_mut().enumerate() {
            // the radio is not defined yet, use it
            if *radio_freq == 0 {
                *radio_freq = min_radio_center_freq;
                break;
            }

            // channel fits within bandwidth of radio
            if channel_max <= *radio_freq + (radio_bw / 2) {
                break;
            }

            // the channel does not fit
            if i == (config.gateway.model_config.radio_count - 1) {
                return Err(
                    "the channels do not fit within the bandwidth of the two radios".to_string(),
                );
            }
        }
    }

    return Ok(radios);
}

pub fn get_radio_for_channel(
    radios: &[u32],
    freq_hz: u32,
    bandwidth: u32,
) -> Result<usize, String> {
    let chan_min = freq_hz - (bandwidth / 2);
    let chan_max = freq_hz + (bandwidth / 2);

    let radio_bandwidth = get_radio_bandwidth(bandwidth);
    for (i, radio_freq) in radios.iter().enumerate() {
        if chan_min >= radio_freq - (radio_bandwidth / 2)
            && chan_max <= radio_freq + (radio_bandwidth / 2)
        {
            return Ok(i);
        }
    }

    return Err("channel does not fit in radio bandwidth".to_string());
}

#[cfg(test)]
mod tests {
    use super::super::vendor::Configuration;
    use libloragw_sx1301::hal;

    #[test]
    fn test_get_radio_frequencies() {
        let tests = vec![
            (
                "EU one channel".to_string(),
                [868100000, 0, 0, 0, 0, 0, 0, 0], // multi SF
                0,                                // LoRa std freq
                0,                                // LoRa std bw
                0,                                // FSK freq
                0,                                // FSK bw
                [868500000, 0],                   // expected
            ),
            (
                "EU three channels".to_string(),
                [868100000, 868300000, 868500000, 0, 0, 0, 0, 0],
                0, // LoRa std freq
                0, // LoRa std bw
                0, // FSK freq
                0, // FSK bw
                [868500000, 0],
            ),
            (
                "EU 8 channels + single SF + FSK".to_string(),
                [
                    868100000, 868300000, 868500000, 867100000, 867300000, 867500000, 867700000,
                    867900000,
                ],
                868300000,
                250000,
                868000000,
                125000,
                [867500000, 868500000],
            ),
            (
                "US915 0-7 + 64".to_string(),
                [
                    902300000, 902500000, 902700000, 902900000, 903100000, 903300000, 903500000,
                    903700000,
                ],
                903000000,
                500000,
                0,
                0,
                [902700000, 903700000],
            ),
        ];

        for (_, multi_sf_channels, lora_std_freq, lora_std_bw, fsk_freq, fsk_bw, expected) in tests
        {
            let mut config: super::super::Configuration = Default::default();

            config.gateway.concentrator.multi_sf_channels = multi_sf_channels;
            config.gateway.concentrator.lora_std.frequency = lora_std_freq;
            config.gateway.concentrator.lora_std.bandwidth = lora_std_bw;
            config.gateway.concentrator.fsk.frequency = fsk_freq;
            config.gateway.concentrator.fsk.bandwidth = fsk_bw;
            config.gateway.model_config = Configuration {
                radio_count: 2,
                clock_source: 1,
                radio_rssi_offset: vec![-166.0, -166.0],
                radio_tx_enabled: vec![true, false],
                radio_type: vec![hal::RadioType::SX1257, hal::RadioType::SX1257],
                radio_min_max_tx_freq: vec![(863000000, 870000000), (863000000, 870000000)],
                radio_tx_notch_freq: vec![0, 0],
                lora_multi_sf_bandwidth: 125000,
                tx_gain_table: vec![
                    hal::TxGainConfig {
                        pa_gain: 0,
                        mix_gain: 8,
                        rf_power: -6,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 0,
                        mix_gain: 10,
                        rf_power: -3,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 0,
                        mix_gain: 12,
                        rf_power: 0,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 1,
                        mix_gain: 8,
                        rf_power: 3,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 1,
                        mix_gain: 10,
                        rf_power: 6,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 1,
                        mix_gain: 12,
                        rf_power: 10,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 1,
                        mix_gain: 13,
                        rf_power: 11,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 2,
                        mix_gain: 9,
                        rf_power: 12,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 1,
                        mix_gain: 15,
                        rf_power: 13,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 2,
                        mix_gain: 10,
                        rf_power: 14,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 2,
                        mix_gain: 11,
                        rf_power: 16,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                    hal::TxGainConfig {
                        pa_gain: 3,
                        mix_gain: 9,
                        rf_power: 20,
                        dig_gain: 0,
                        dac_gain: 3,
                    },
                ],
            };

            let radios = super::get_radio_frequencies(&config).unwrap();
            assert_eq!(radios, expected);
        }
    }

    #[test]
    fn test_get_radio_for_channel() {
        let tests = vec![("Radio 0".to_string(), [868500000, 0], 868100000, 125000, 0)];

        for (_, radios, freq_hz, bandwidth, expected) in tests {
            let radio = super::get_radio_for_channel(&radios, freq_hz, bandwidth).unwrap();
            assert_eq!(radio, expected);
        }
    }
}
