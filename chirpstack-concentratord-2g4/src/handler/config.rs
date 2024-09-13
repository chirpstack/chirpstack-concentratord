use anyhow::Result;

use super::super::config::{Channel, Concentrator, Configuration};

pub fn update_configuration(
    config: &mut Configuration,
    new_config: &chirpstack_api::gw::GatewayConfiguration,
) -> Result<() > {
    info!("Updating concentrator configuration");

    // empty concentrator config
    let mut concentrator = Concentrator::default();

    if new_config.channels.len() > concentrator.channels.len() {
        return Err(anyhow!("configuration exceeds max number of channels"));
    }

    for (i, channel) in new_config.channels.iter().enumerate() {
        match &channel.modulation_config {
            Some(
                chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(
                    v,
                ),
            ) => {
                if v.spreading_factors.len() != 1 {
                    return Err(
                        anyhow!("spreading_factors must contain a single spreading-factor"));
                }

                concentrator.channels[i] = Channel {
                    frequency: channel.frequency,
                    bandwidth: v.bandwidth,
                    spreading_factor: v.spreading_factors[0],
                    rssi_offset: 0.0,
                };
            }
            _ => {
                return Err(anyhow!("channel modulation must be LORA"));
            }
        }
    }

    // set config
    if let Some(stats_interval) = new_config.stats_interval {
        config.concentratord.stats_interval = stats_interval.try_into()?;
    }
    config.gateway.config_version.clone_from(&new_config.version);
    config.gateway.concentrator = concentrator;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::super::config::Channel;
    use super::{update_configuration, Concentrator, Configuration};

    #[test]
    fn test_update_configuration() {
        let tests = vec![(
            "Single channel",
            chirpstack_api::gw::GatewayConfiguration {
                channels: vec![
                    chirpstack_api::gw::ChannelConfiguration{
                        frequency: 2403000000,
                        modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                            bandwidth: 812000,
                            spreading_factors: vec![12],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Concentrator {
                channels: [Channel {
                    frequency: 2403000000,
                    bandwidth: 812000,
                    spreading_factor: 12,
                    rssi_offset: 0.0,
                }, 
                Channel { ..Default::default() },
                Channel { ..Default::default() },
                ],
            },
            ), (
            "Three channels",
            chirpstack_api::gw::GatewayConfiguration {
                channels: vec![
                    chirpstack_api::gw::ChannelConfiguration{
                        frequency: 2403000000,
                        modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                            bandwidth: 812000,
                            spreading_factors: vec![12],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    chirpstack_api::gw::ChannelConfiguration{
                        frequency: 2479000000,
                        modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                            bandwidth: 812000,
                            spreading_factors: vec![12],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    chirpstack_api::gw::ChannelConfiguration{
                        frequency: 2425000000,
                        modulation_config: Some(chirpstack_api::gw::channel_configuration::ModulationConfig::LoraModulationConfig(chirpstack_api::gw::LoraModulationConfig{
                            bandwidth: 812000,
                            spreading_factors: vec![12],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Concentrator {
                channels: [
                    Channel {
                        frequency: 2403000000,
                        bandwidth: 812000,
                        spreading_factor: 12,
                        rssi_offset: 0.0,
                    },
                    Channel {
                        frequency: 2479000000,
                        bandwidth: 812000,
                        spreading_factor: 12,
                        rssi_offset: 0.0,
                    }, 
                    Channel {
                        frequency: 2425000000,
                        bandwidth: 812000,
                        spreading_factor: 12,
                        rssi_offset: 0.0,
                    }, 
                ],
            },
        )];

        for (_, gw_config, expected_config) in tests {
            let mut config = Configuration::default();
            update_configuration(&mut config, &gw_config).unwrap();
            assert_eq!(config.gateway.concentrator, expected_config);
        }
    }
}
