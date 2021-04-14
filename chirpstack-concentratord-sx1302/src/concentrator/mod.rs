use libloragw_sx1302::{com, hal};

use super::config::vendor::ComType;
use super::config::{helpers, Configuration};

pub fn board_setconf(config: &Configuration) -> Result<(), String> {
    let board_config = hal::BoardConfig {
        lorawan_public: config.gateway.lorawan_public,
        clock_source: config.gateway.model_config.clock_source,
        full_duplex: config.gateway.model_config.full_duplex,
        com_type: match config.gateway.model_config.com_type {
            ComType::SPI => com::ComType::SPI,
            ComType::USB => com::ComType::USB,
        },
        com_path: config.gateway.model_config.com_path.clone(),
    };

    info!(
        "Setting board configuration, lorawan_public: {}, clock_source: {}",
        board_config.lorawan_public, board_config.clock_source
    );
    hal::board_setconf(&board_config)?;

    return Ok(());
}

pub fn timestamp_setconf(config: &Configuration) -> Result<(), String> {
    info!(
        "Setting up fine timestamp, enable: {}",
        config.gateway.fine_timestamp.enable
    );
    let ts_config = hal::TimestampConfig {
        enable: config.gateway.fine_timestamp.enable,
        mode: match config.gateway.fine_timestamp.mode.as_ref() {
            "HIGH_CAPACITY" => hal::FineTimestampMode::HighCapacity,
            "ALL_SF" => hal::FineTimestampMode::AllSF,
            _ => {
                return Err("fine_timestamp mode must be HIGH_CAPACITY or ALL_SF".to_string());
            }
        },
    };
    hal::ftime_setconf(&ts_config)?;
    return Ok(());
}

pub fn txgain_setconf(config: &Configuration) -> Result<(), String> {
    for (i, radio_config) in config.gateway.model_config.radio_config.iter().enumerate() {
        if radio_config.tx_gain_table.len() == 0 {
            continue;
        }

        for tx_gain_config in &radio_config.tx_gain_table {
            debug!("Configuration TX gain for radio, radio: {}, rf_power: {}, dig_gain: {}, pa_gain: {}, dac_gain: {}, mix_gain: {}, offset_i: {}, offset_q: {}, pwr_idx: {}", i, tx_gain_config.rf_power, tx_gain_config.dig_gain, tx_gain_config.pa_gain, tx_gain_config.dac_gain, tx_gain_config.mix_gain, tx_gain_config.offset_i, tx_gain_config.offset_q, tx_gain_config.pwr_idx);
        }

        hal::txgain_setconf(i as u8, &radio_config.tx_gain_table)?;
    }

    return Ok(());
}

pub fn rxrf_setconf(config: &Configuration) -> Result<(), String> {
    info!("Setting up concentrator channels");
    let radio_freqs = helpers::get_radio_frequencies(&config)?;
    for (i, radio_freq) in radio_freqs.iter().enumerate() {
        let rx_rf_config = hal::RxRfConfig {
            enable: *radio_freq > 0,
            freq_hz: *radio_freq,
            rssi_offset: config.gateway.model_config.radio_config[i].rssi_offset,
            rssi_temp_compensation: config.gateway.model_config.radio_config[i]
                .rssi_temp_compensation,
            radio_type: config.gateway.model_config.radio_config[i].radio_type,
            tx_enable: config.gateway.model_config.radio_config[i].tx_enable,
            single_input_mode: config.gateway.model_config.radio_config[i].single_input_mode,
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

    // LoRa mult-SF
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
        }

        info!("Configuring multi-SF LoRa channel, channel: {}, enabled: {}, freq: {}, rf_chain: {}, if_freq: {}", i, rx_if_config.enable, chan_freq, rx_if_config.rf_chain, rx_if_config.freq_hz);
        hal::rxif_setconf(i as u8, &rx_if_config)?;
    }

    // LoRa Std
    let mut rx_if_config: hal::RxIfConfig = Default::default();
    let lora_std = &config.gateway.concentrator.lora_std;
    rx_if_config.enable = lora_std.frequency > 0;
    if rx_if_config.enable {
        let chan_radio =
            helpers::get_radio_for_channel(&radio_freqs, lora_std.frequency, lora_std.bandwidth)
                .unwrap();

        rx_if_config.rf_chain = chan_radio as u8;
        rx_if_config.freq_hz = lora_std.frequency as i32 - radio_freqs[chan_radio] as i32;
        rx_if_config.bandwidth = lora_std.bandwidth;
        rx_if_config.datarate = match lora_std.spreading_factor {
            5 => hal::DataRate::SF5,
            6 => hal::DataRate::SF6,
            7 => hal::DataRate::SF7,
            8 => hal::DataRate::SF8,
            9 => hal::DataRate::SF9,
            10 => hal::DataRate::SF10,
            11 => hal::DataRate::SF11,
            12 => hal::DataRate::SF12,
            _ => hal::DataRate::Undefined,
        };
        rx_if_config.implicit_header = lora_std.implicit_header;

        if rx_if_config.implicit_header {
            rx_if_config.implicit_payload_length = lora_std.implicit_payload_length;
            rx_if_config.implicit_crc_enable = lora_std.implicit_crc_enable;
            rx_if_config.implicit_coderate = match lora_std.implicit_coderate.as_ref() {
                "4/5" => hal::CodeRate::LoRa4_5,
                "4/6" => hal::CodeRate::LoRa4_6,
                "4/7" => hal::CodeRate::LoRa4_7,
                "4/8" => hal::CodeRate::LoRa4_8,
                _ => return Err("invalid implicit_coderate".to_string()),
            };
        }
    }
    info!(
        "Configuring Std LoRa channel, enabled: {}, freq: {}, rf_chain: {}, if_freq: {}",
        rx_if_config.enable, lora_std.frequency, rx_if_config.rf_chain, rx_if_config.freq_hz
    );
    hal::rxif_setconf(8, &rx_if_config)?;

    // FSK
    let mut rx_if_config: hal::RxIfConfig = Default::default();
    let fsk = &config.gateway.concentrator.fsk;
    rx_if_config.enable = fsk.frequency > 0;
    if rx_if_config.enable {
        let chan_radio =
            helpers::get_radio_for_channel(&radio_freqs, fsk.frequency, fsk.bandwidth).unwrap();

        rx_if_config.rf_chain = chan_radio as u8;
        rx_if_config.freq_hz = fsk.frequency as i32 - radio_freqs[chan_radio] as i32;
        rx_if_config.datarate = hal::DataRate::FSK(fsk.datarate);
        rx_if_config.bandwidth = fsk.bandwidth;
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

pub fn start() -> Result<(), String> {
    info!("Starting the concentrator");
    hal::start()?;

    return Ok(());
}

pub fn stop() -> Result<(), String> {
    info!("Stopping the concentrator");
    hal::stop()?;

    return Ok(());
}

pub fn get_eui() -> Result<[u8; 8], String> {
    debug!("Getting gateway EUI");
    return hal::get_eui();
}
