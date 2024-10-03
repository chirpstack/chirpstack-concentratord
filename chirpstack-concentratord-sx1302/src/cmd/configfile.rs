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

  # Disable CRC status filter.
  #
  # By default, the Concentratord will ignore received frames which do not have
  # a valid CRC. This option makes it possible to disable this filter such that
  # received frames without (valid) CRC can be analyzed.
  disable_crc_filter={{ concentratord.disable_crc_filter }}

  # Configuration for the (ZeroMQ based) API.
  [concentratord.api]
    # Event PUB socket bind.
    event_bind="{{ concentratord.api.event_bind }}"

    # Command REP socket bind.
    command_bind="{{ concentratord.api.command_bind }}"


# LoRa gateway configuration.
[gateway]

  # Antenna gain (dBi).
  antenna_gain={{ gateway.antenna_gain }}

  # Public LoRaWAN network.
  lorawan_public={{ gateway.lorawan_public }}

  # Region.
  #
  # The region of the gateway. Options:
  #  EU868, US915, CN779, EU433, AU915, CN470, AS923, AS923_2, AS923_3, AS923_4,
  #  KR923, IN865, RU864
  #
  # Not not all the gateway models implement all regions.
  region="{{ gateway.region }}"

  # Gateway vendor / model.
  #
  # This configures various vendor and model specific settings.
  model="{{ gateway.model }}"

  # Gateway vendor / model flags.
  #
  # Flag can be used to configure additional vendor / model features. The
  # following flags can be used:
  #
  #   Global flags:
  #     GNSS - Enable GNSS / GPS support
  #     USB  - Use USB for concentrator communication (default is SPI)
  model_flags=[{{#each gateway.model_flags}}"{{ this }},{{/each}}]

  # Time fallback.
  #
  # In case the gateway does not have a GNSS module or is unable to aquire a
  # GNSS fix, use the system-time for setting the 'time' field on RX.
  time_fallback_enabled={{ gateway.time_fallback_enabled }}

  # Device and pin configuration.
  #
  # Only set this configuration if you need to override the default
  # configuration provided by the model configuration and you know
  # the device and pin mapping. In any other case, leave this commented
  # out.
  # gnss_dev_path="/dev/ttyAMA0"
  # gnss_dev_path="gpsd://localhost:2947"
  # com_dev_path="/dev/spidev0.0"
  # i2c_dev_path="/dev/i2c-1"
  # sx1302_reset_chip="/dev/gpiochip0"
  # sx1302_reset_pin=17
  # sx1302_power_en_chip="/dev/gpiochip0"
  # sx1302_power_en_pin=18
  # sx1261_reset_chip="/dev/gpiochip0"
  # sx1261_reset_pin=19


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
