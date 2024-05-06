use std::cmp::Ordering;

use anyhow::Result;

use super::super::config::{Concentrator, Configuration};

pub fn update_configuration(
    config: &mut Configuration,
    new_config: &chirpstack_api::gw::GatewayConfiguration,
) -> Result<()> {
    info!("Updating concentrator configuration");

    // empty concentrator config
    let mut concentrator = Concentrator::default();
    let mut multi_sf_count = 0;

    for channel in new_config.channels.iter() {
        match &channel.modulation_config {
            Some(
                chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(
                    v,
                ),
            ) => match v.spreading_factors.len().cmp(&1) {
                Ordering::Equal => {
                    concentrator.lora_std.frequency = channel.frequency;
                    concentrator.lora_std.bandwidth = v.bandwidth;
                    concentrator.lora_std.spreading_factor = v.spreading_factors[0] as u8;
                }
                Ordering::Greater => {
                    if multi_sf_count > concentrator.multi_sf_channels.len() - 1 {
                        return Err(anyhow!("too many multi-SF channels in configuration"));
                    }

                    concentrator.multi_sf_channels[multi_sf_count] = channel.frequency;
                    multi_sf_count += 1;
                }
                _ => {}
            },
            Some(
                chirpstack_api::gw::channel_configuration::ModulationConfig::FskModulationConfig(v),
            ) => {
                concentrator.fsk.frequency = channel.frequency;
                concentrator.fsk.bandwidth = v.bandwidth;
                concentrator.fsk.datarate = v.bitrate;
            }
            None => {}
        };
    }

    // set config
    config
        .gateway
        .config_version
        .clone_from(&new_config.version);
    config.gateway.concentrator = concentrator;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::super::config::{FSKChannel, LoRaStdChannel};
    use super::{update_configuration, Concentrator, Configuration};

    #[test]
    fn test_update_configuration() {
        let tests = vec![
            (
                "Single channel",
                chirpstack_api::gw::GatewayConfiguration{
                    channels: vec![
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 868100000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                Concentrator{
                    multi_sf_channels: [
                        868100000,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                    ],
                    ..Default::default()
                },
            ),
            (
                "8 channels + LoRa std + FSK",
                chirpstack_api::gw::GatewayConfiguration{
                    channels: vec![
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 868100000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 868300000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 868500000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 867100000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 867300000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 867500000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 867700000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 867900000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 125000,
                                spreading_factors: vec![7,8,9,10,11,12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 868300000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                                bandwidth: 250000,
                                spreading_factors: vec![12],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        chirpstack_api::gw::ChannelConfiguration{
                            frequency: 868800000,
                            modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::FskModulationConfig(chirpstack_api::gw::FskModulationConfig{
                                bandwidth: 125000,
                                bitrate: 50000,
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                Concentrator{
                    multi_sf_channels: [
                        868100000,
                        868300000,
                        868500000,
                        867100000,
                        867300000,
                        867500000,
                        867700000,
                        867900000,
                    ],
                    lora_std: LoRaStdChannel{
                        frequency: 868300000,
                        bandwidth: 250000,
                        spreading_factor: 12,
                    },
                    fsk: FSKChannel{
                        frequency: 868800000,
                        datarate: 50000,
                        bandwidth: 125000,
                    },
                },
            ),
        ];

        for (_, gw_config, expected_config) in tests {
            let mut config = Configuration::default();
            update_configuration(&mut config, &gw_config).unwrap();
            assert_eq!(config.gateway.concentrator, expected_config);
        }
    }
}
