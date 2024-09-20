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

  # Gateway vendor / model.
  #
  # This configures various vendor and model specific settings like the min / max
  # frequency and TX gain table.
  model="{{ gateway.model }}"

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
  # com_dev_path="/dev/ttyACM0"
  # mcu_reset_chip="/dev/gpiochip0"
  # mcu_reset_pin=32
  # mcu_boot_chip="/dev/gpiochip0"
  # mcu_boot_pin=18


  # LoRa concentrator configuration.
  [gateway.concentrator]
{{#each gateway.concentrator.channels}}
    [[gateway.concentrator.channels]]
      frequency={{ this.frequency }}
      bandwidth={{ this.bandwidth }}
      spreading_factor={{ this.spreading_factor }}
      rssi_offset={{ this.rssi_offset }}
{{/each}}


  # Static gateway location.
  [gateway.location]
    # If set to non-zero values, the static gateway location will be reported
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
