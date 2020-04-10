use libloragw_sx1301::{hal, spi};

use super::config::{helpers, Configuration};

pub fn set_spidev_path(config: &Configuration) -> Result<(), String> {
    info!(
        "Setting spi device path, spidev_path: {}",
        config.gateway.model_config.spidev_path
    );
    spi::set_path(&config.gateway.model_config.spidev_path)?;

    return Ok(());
}

pub fn board_setconf(config: &Configuration) -> Result<(), String> {
    let board_config = hal::BoardConfig {
        lorawan_public: config.gateway.lorawan_public,
        clock_source: config.gateway.model_config.clock_source,
    };
    info!(
        "Setting board configuration, lorawan_public: {}, clock_source: {}",
        board_config.lorawan_public, board_config.clock_source
    );
    hal::board_setconf(&board_config)?;

    return Ok(());
}

pub fn txgain_setconf(config: &Configuration) -> Result<(), String> {
    for tx_gain_config in &config.gateway.model_config.tx_gain_table {
        debug!("Configuring TX gain, rf_power: {}, pa_gain: {}, mix_gain: {}, dac_gain: {}, dig_gain: {}", tx_gain_config.rf_power, tx_gain_config.pa_gain, tx_gain_config.mix_gain,  tx_gain_config.dac_gain, tx_gain_config.dig_gain);
    }
    hal::txgain_setconf(&config.gateway.model_config.tx_gain_table)?;

    return Ok(());
}

pub fn rxrf_setconf(config: &Configuration) -> Result<(), String> {
    info!("Setting up concentrator radios");
    let radio_freqs = helpers::get_radio_frequencies(&config)?;
    for (i, radio_freq) in radio_freqs.iter().enumerate() {
        let rx_rf_config = hal::RxRfConfig {
            enable: *radio_freq > 0,
            freq_hz: *radio_freq,
            rssi_offset: config.gateway.model_config.radio_rssi_offset[i],
            radio_type: config.gateway.model_config.radio_type[i],
            tx_enable: config.gateway.model_config.radio_tx_enabled[i],
            tx_notch_freq: config.gateway.model_config.radio_tx_notch_freq[i],
        };

        info!(
            "Configuring radio, radio: {}, enabled: {}, center_freq: {}, type: {:?}",
            i, rx_rf_config.enable, rx_rf_config.freq_hz, rx_rf_config.radio_type
        );
        hal::rxrf_setconf(i as u8, &rx_rf_config)?;
    }

    return Ok(());
}

pub fn rxif_setconf(config: &Configuration) -> Result<(), String> {
    info!("Setting up concentrator channels");
    let radio_freqs = helpers::get_radio_frequencies(&config)?;
    for (i, chan_freq) in config
        .gateway
        .concentrator
        .multi_sf_channels
        .iter()
        .enumerate()
    {
        let mut rx_if_config: hal::RxIfConfig = Default::default();
        rx_if_config.enable = *chan_freq > 0;

        if rx_if_config.enable {
            let chan_radio = helpers::get_radio_for_channel(
                &radio_freqs,
                *chan_freq,
                config.gateway.model_config.lora_multi_sf_bandwidth,
            )
            .unwrap();

            rx_if_config.rf_chain = chan_radio as u8;
            rx_if_config.freq_hz = *chan_freq as i32 - radio_freqs[chan_radio] as i32;
            rx_if_config.bandwidth = config.gateway.model_config.lora_multi_sf_bandwidth;
            rx_if_config.datarate = hal::DataRate::MultiSF;
        }

        info!("Configuring multi-SF LoRa channel, channel: {}, enabled: {}, freq: {}, rf_chain: {}, if_freq: {}", i, rx_if_config.enable, chan_freq, rx_if_config.rf_chain, rx_if_config.freq_hz);
        hal::rxif_setconf(i as u8, &rx_if_config)?;
    }

    let mut rx_if_config: hal::RxIfConfig = Default::default();
    rx_if_config.enable = config.gateway.concentrator.lora_std.frequency > 0;

    if rx_if_config.enable {
        let chan_radio = helpers::get_radio_for_channel(
            &radio_freqs,
            config.gateway.concentrator.lora_std.frequency,
            config.gateway.concentrator.lora_std.bandwidth,
        )
        .unwrap();

        rx_if_config.rf_chain = chan_radio as u8;
        rx_if_config.freq_hz =
            config.gateway.concentrator.lora_std.frequency as i32 - radio_freqs[chan_radio] as i32;
        rx_if_config.bandwidth = config.gateway.concentrator.lora_std.bandwidth;
        rx_if_config.datarate = match config.gateway.concentrator.lora_std.spreading_factor {
            7 => hal::DataRate::SF7,
            8 => hal::DataRate::SF8,
            9 => hal::DataRate::SF9,
            10 => hal::DataRate::SF10,
            11 => hal::DataRate::SF11,
            12 => hal::DataRate::SF12,
            _ => hal::DataRate::Undefined,
        };
    }
    info!(
        "Configuring Std LoRa channel, enabled: {}, freq: {}, rf_chain: {}, if_freq: {}",
        rx_if_config.enable,
        config.gateway.concentrator.lora_std.frequency,
        rx_if_config.rf_chain,
        rx_if_config.freq_hz
    );
    hal::rxif_setconf(8, &rx_if_config)?;

    let mut rx_if_config: hal::RxIfConfig = Default::default();
    rx_if_config.enable = config.gateway.concentrator.fsk.frequency > 0;

    if rx_if_config.enable {
        let chan_radio = helpers::get_radio_for_channel(
            &radio_freqs,
            config.gateway.concentrator.fsk.frequency,
            config.gateway.concentrator.fsk.bandwidth,
        )
        .unwrap();

        rx_if_config.rf_chain = chan_radio as u8;
        rx_if_config.freq_hz =
            config.gateway.concentrator.fsk.frequency as i32 - radio_freqs[chan_radio] as i32;
        rx_if_config.bandwidth = config.gateway.concentrator.fsk.bandwidth;
        rx_if_config.datarate = hal::DataRate::FSK(config.gateway.concentrator.fsk.datarate);
    }
    info!(
        "Configuring FSK channel, enabled: {}, freq: {}, rf_chain: {}, if_freq: {}",
        rx_if_config.enable,
        config.gateway.concentrator.fsk.frequency,
        rx_if_config.rf_chain,
        rx_if_config.freq_hz
    );
    hal::rxif_setconf(9, &rx_if_config)?;

    return Ok(());
}

pub fn start(_config: &Configuration) -> Result<(), String> {
    info!("Starting the concentrator");
    hal::start()?;

    return Ok(());
}

pub fn stop(_config: &Configuration) -> Result<(), String> {
    info!("Stopping the concentrator");
    hal::stop()?;

    return Ok(());
}
