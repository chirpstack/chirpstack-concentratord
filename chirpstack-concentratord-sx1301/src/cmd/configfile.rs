use handlebars::Handlebars;

use super::super::config;

pub fn run(config: &config::Configuration) {
    let template = r#"
# Concentratord configuration.
[concentratord]
  # Log level.
  #
  # Valid options are:
  #   * TRACE
  #   * DEBUG
  #   * INFO
  #   * WARN
  #   * ERROR
  #   * OFF
  log_level="{{ concentratord.log_level }}"

  # Log to syslog.
  #
  # When set to true, log messages are being written to syslog instead of stdout.
  log_to_syslog={{ concentratord.log_to_syslog }}

  # Statistics interval.
  stats_interval="{{ concentratord.stats_interval }}"

  # Configuration for the (ZeroMQ based) API.
  [concentratord.api]
    # Event PUB socket bind.
    event_bind="{{ concentratord.api.event_bind }}"

    # Command REP socket bind.
    command_bind="{{ concentratord.api.command_bind }}"


# LoRa gateway configuration.
[gateway]

  # Antenna gain (dB).
  antenna_gain={{ gateway.antenna_gain }}

  # Public LoRaWAN network.
  lorawan_public={{ gateway.lorawan_public }}

  # Gateway vendor / model.
  #
  # This configures various vendor and model specific settings like the min / max
  # frequency and TX gain table.
  model="{{ gateway.model }}"

  # Gateway vendor / model flags.
  #
  # Flag can be used to configure additional vendor / model features. The
  # following flags can be used:
  #
  #   Global flags:
  #     GNSS - Enable GNSS / GPS support
  #
  #   Multitech:
  #     AP1  - Module is in AP1 slot (default)
  #     AP2  - Module is in AP2 slot
  model_flags=[{{#each gateway.model_flags}}"{{ this }}",{{/each}}]

  # Gateway ID.
  gateway_id="{{ gateway.gateway_id }}"


  # LoRa concentrator configuration.
  [gateway.concentrator]

    # Multi spreading-factor channels (LoRa).
    multi_sf_channels=[{{#each gateway.concentrator.multi_sf_channels}}
      {{ this }},{{/each}}
    ]

    # LoRa std channel (single spreading-factor).
    [gateway.concentrator.lora_std]
      frequency={{ gateway.concentrator.lora_std.frequency }}
      bandwidth={{ gateway.concentrator.lora_std.bandwidth }}
      spreading_factor={{ gateway.concentrator.lora_std.spreading_factor }}

    # FSK channel.
    [gateway.concentrator.fsk]
      frequency={{ gateway.concentrator.fsk.frequency }}
      bandwidth={{ gateway.concentrator.fsk.bandwidth }}
      datarate={{ gateway.concentrator.fsk.datarate }}


  # Beacon configuration.
  #
  # This requires a gateway with GPS / GNSS.
  #
  # Please note that the beacon settings are region dependent. The correct
  # settings can be found in the LoRaWAN Regional Parameters specification.
  [gateway.beacon]

    # Compulsory RFU size.
    compulsory_rfu_size={{ gateway.beacon.compulsory_rfu_size }}

    # Beacon frequency / frequencies (Hz).
    frequencies=[{{#each gateway.beacon.frequencies}}
      {{ this }},{{/each}}
    ]

    # Bandwidth (Hz).
    bandwidth={{ gateway.beacon.bandwidth }}

    # Spreading factor.
    spreading_factor={{ gateway.beacon.spreading_factor }}

    # TX power.
    tx_power={{ gateway.beacon.tx_power }}


  # Static gateway location.
  [gateway.location]

    # When set to non-zero values, the static gateway location will be reported
    # when the gateway does not have a GNSS module or when no GNSS location fix
    # is available.
    latitude={{ gateway.location.latitude }}
    longitude={{ gateway.location.longitude }}
    altitude={{ gateway.location.altitude }}
"#;

    let reg = Handlebars::new();
    println!(
        "{}",
        reg.render_template(template, config)
            .expect("render configfile error")
    );
}
