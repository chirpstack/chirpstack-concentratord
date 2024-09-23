use anyhow::Result;
use libloragw_2g4::hal;

use super::config::Configuration;

pub fn board_setconf(config: &Configuration) -> Result<()> {
    let board_config = hal::BoardConfig {
        tty_path: config.gateway.model_config.tty_path.clone(),
    };

    info!("Setting board configuration");
    hal::board_setconf(&board_config)?;

    Ok(())
}

pub fn rx_setconf(config: &Configuration) -> Result<()> {
    info!("Setting up concentrator channels");

    for (i, channel) in config.gateway.concentrator.channels.iter().enumerate() {
        info!(
            "Configuring channel: {}, center_freq: {}",
            i, channel.frequency
        );

        hal::channel_rx_setconf(
            i as u8,
            &hal::ChannelRxConfig {
                enable: channel.frequency > 0,
                freq_hz: channel.frequency,
                bandwidth: channel.bandwidth,
                datarate: match channel.spreading_factor {
                    5 => hal::DataRate::SF5,
                    6 => hal::DataRate::SF6,
                    7 => hal::DataRate::SF7,
                    8 => hal::DataRate::SF8,
                    9 => hal::DataRate::SF9,
                    10 => hal::DataRate::SF10,
                    11 => hal::DataRate::SF11,
                    12 => hal::DataRate::SF12,
                    _ => return Err(anyhow!("invalid spreading_factor")),
                },
                rssi_offset: channel.rssi_offset,
                sync_word: match config.gateway.lorawan_public {
                    true => 0x21,
                    false => 0x12,
                },
            },
        )?;
    }

    Ok(())
}

pub fn tx_setconf(config: &Configuration) -> Result<()> {
    let enable = !config.gateway.model_config.tx_min_max_freqs.is_empty();
    info!("Configuring tx path, enable: {}", enable);
    hal::channel_tx_setconf(&hal::ChannelTxConfig { enable })
}

pub fn start() -> Result<()> {
    info!("Starting the concentrator");
    hal::start()
}

pub fn stop() -> Result<()> {
    info!("Stopping the concentrator");
    hal::stop()
}

pub fn get_eui() -> Result<[u8; 8]> {
    debug!("Getting the gateway EUI");
    hal::get_eui()
}
