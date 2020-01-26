# ChirpStack Concentratord

ChirpStack Concentratord is an open-source LoRa(WAN) concentrator daemon and is
part of the [ChirpStack](https://www.chirpstack.io/) project.
It exposes a common [ZeroMQ](https://zeromq.org/) based api that can be used by LoRa
Packet Forwarders for interacting with gateway hardware. By implementing the
hardware specifics in a separate daemon, a LoRa Packet Forwarder will work with
any supported gateway as long as the Concentratord protocol is implemented.

**Note:** This is an experimental project!

## Events

Events are published by Concentratord and can be received by creating a
[ZeroMQ SUB](http://zguide.zeromq.org/page:all#toc49) socket. The first frame
holds the event type (string), the second frame holds the event payload encoded
using [Protobuf](https://developers.google.com/protocol-buffers)
(see `protobuf/gw/gw.proto` in [chirpstack-api](https://github.com/brocaar/chirpstack-api)
for the Protobuf message definitions).

### `up`

Uplink received by the gateway (`UplinkFrame` Protobuf message). 

### `stats`

Gateway statistics (`GatewayStats` Protobuf message).

## Commands

Commands can be sent to Concentratord using a [ZeroMQ REQ](http://zguide.zeromq.org/page:all#toc52)
socket. The first data-frame holds the command type (string), the second
data-frame holds the command payload encoded using Protobuf.

### `down`

Request to enqueue the given frame for downlink (`DownlinkFrame` Protobuf
message). A downlink command is responded by a `DownlinkTXAck` message.

### `gateway_id`

Request the Gateway ID (the data-frame is empty). The response is the 8byte
Gateway ID.

## Supported hardware

### chirpstack-concentratord-sx1301

The `chirpstack-concentratord-sx1301` implements the [SX1301 HAL](https://github.com/lora-net/lora_gateway).
It has been tested on Raspberry Pi based gateways.

#### Configuration

Configuration example:

```toml
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
log_level="INFO"

# Log to syslog.
#
# When set to true, log messages are being written to syslog instead of stdout.
log_to_syslog=false

# Statistics interval.
stats_interval="30s"

  # Configuration for the (ZeroMQ based) API.
  [concentratord.api]

  # Event PUB socket bind.
  event_bind="ipc:///tmp/concentratord_event"

  # Command REP socket bind.
  command_bind="ipc:///tmp/concentratord_command"


# LoRa gateway configuration.
[gateway]

# Antenna gain (dB).
antenna_gain=0

# Public LoRaWAN network.
lorawan_public=true

# Gateway vendor / model.
#
# This configures various vendor and model specific settings like the min / max
# frequency, TX gain table, ... Valid options are:
#   * generic_as923      - Generic AS923 model
#   * generic_as923_gps  - Generic AS923 model (with GPS)
#   * generic_au915      - Generic AU915 model
#   * generic_au915_gps  - Generic AU915 model (with GPS)
#   * generic_cn470      - Generic CN470 model
#   * generic_cn470_gps  - Generic CN470 model (with GPS)
#   * generic_eu868      - Generic EU868 model
#   * generic_eu868_gps  - Generic EU868 model (with GPS)
#   * generic_in865      - Generic IN865 model
#   * generic_in865_gps  - Generic IN865 model (with GPS)
#   * generic_kr920      - Generic KR920 model
#   * generic_kr920_gps  - Generic KR920 model (with GPS)
#   * generic_ru864      - Generic RU864 model
#   * generic_ru864_gps  - Generic RU864 model (with GPS)
#   * generic_us915      - Generic US915 model
#   * generic_us915_gps  - Generic US915 model (with GPS)
#   * multitech_mtac_lora_h_868_eu868_ap1      - Multitech Conduit - MTAC-LORA-H-868 - EU868 - AP1 slot
#   * multitech_mtac_lora_h_868_eu868_ap1_gps  - Multitech Conduit - MTAC-LORA-H-868 - EU868 - AP1 slot (with GPS)
#   * multitech_mtac_lora_h_868_eu868_ap2      - Multitech Conduit - MTAC-LORA-H-868 - EU868 - AP2 slot
#   * multitech_mtac_lora_h_868_eu868_ap2_gps  - Multitech Conduit - MTAC-LORA-H-868 - EU868 - AP2 slot (with GPS)
#   * multitech_mtac_lora_h_915_us915_ap1      - Multitech Conduit - MTAC-LORA-H-915 - US915 - AP1 slot
#   * multitech_mtac_lora_h_915_us915_ap1_gps  - Multitech Conduit - MTAC-LORA-H-915 - US915 - AP1 slot (with GPS)
#   * multitech_mtac_lora_h_915_us915_ap2      - Multitech Conduit - MTAC-LORA-H-915 - US915 - AP2 slot
#   * multitech_mtac_lora_h_915_us915_ap2_gps  - Multitech Conduit - MTAC-LORA-H-915 - US915 - AP2 slot (with GPS)
model="generic_eu868"

# Gateway ID.
gateway_id="0202030405060708"

  # LoRa concentrator configuration.
  [gateway.concentrator]

  # Multi spreading-factor channels (LoRa).
  multi_sf_channels=[
    868100000,
    868300000,
    868500000,
    867100000,
    867300000,
    867500000,
    867700000,
    867900000,
  ]

  # LoRa std channel (single spreading-factor).
  [gateway.concentrator.lora_std]
  frequency=868300000
  bandwidth=250000
  spreading_factor=7

  # FSK channel.
  [gateway.concentrator.fsk]
  frequency=868800000
  bandwidth=125000
  datarate=50000


  # Beacon configuration.
  #
  # This requires a gateway with GPS / GNSS.
  #
  # Please note that the beacon settings are region dependent. The correct
  # settings can be found in the LoRaWAN Regional Parameters specification.
  [gateway.beacon]

  # Compulsory RFU size.
  compulsory_rfu_size=2

  # Beacon frequency / frequencies (Hz).
  frequencies=[
  	869525000,
  ]

  # Bandwidth (Hz).
  bandwidth=125000

  # Spreading factor.
  spreading_factor=9

  # TX power.
  tx_power=14
```

### chirpstack-concentratord-sx1302

The `chirpstack-concentratord-sx1302` implements the [SX1302 HAL](https://github.com/lora-net/sx1302_hal).
It has been tested on a Raspberry Pi 3 with the Semtech SX1302 CoreCell.

#### Configuration

Configuration example:

```toml
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
log_level="DEBUG"

# Log to syslog.
#
# When set to true, log messages are being written to syslog instead of stdout.
log_to_syslog=false

# Statistics interval.
stats_interval="30s"

  # Configuration for the (ZeroMQ based) API.
  [concentratord.api]

  # Event PUB socket bind.
  event_bind="ipc:///tmp/concentratord_event"

  # Command REP socket bind.
  command_bind="ipc:///tmp/concentratord_command"


# LoRa gateway configuration.
[gateway]

# Antenna gain (dB).
antenna_gain=0

# Public LoRaWAN network.
lorawan_public=true

# Gateway vendor / model.
#
# This configures various vendor and model specific settings like the min / max
# frequency, TX gain table, ... Valid options are:
#   * generic_sx1250_eu868      - Generic SX1250 based EU868 model
#   * generic_sx1250_eu868_gps  - Generic SX1250 based EU868 model (with GPS)
#   * generic_sx1250_us915      - Generic SX1250 based US915 model
#   * generic_sx1250_us915_gps  - Generic SX1250 based US915 model (with GPS)
model="generic_sx1250_eu868"

  # LoRa concentrator configuration.
  [gateway.concentrator]

  # Multi spreading-factor channels (LoRa).
  multi_sf_channels=[
    868100000,
    868300000,
    868500000,
    867100000,
    867300000,
    867500000,
    867700000,
    867900000,
  ]

  # LoRa std channel (single spreading-factor).
  [gateway.concentrator.lora_std]
  frequency=868300000
  bandwidth=250000
  spreading_factor=7

  # FSK channel.
  [gateway.concentrator.fsk]
  frequency=868800000
  bandwidth=125000
  datarate=50000
```

## Building from source

You must have [Docker](https://docs.docker.com/install/) and [Docker Compose](https://docs.docker.com/compose/install/)
installed for these instructions.

```bash
# enter development environment
docker-compose run --rm chirpstack-concentratord bash
```

```bash
# Compile ARMv5 binary
make build-armv5-release

# Compile ARMv7hf binary
make build-armv7hf-release
```

You will find the binary under `target/{ARCHITECTURE}/release`.

### Compile optimizations

The provided `...-release` commands are using the default Rust `release`
mode profile. Several options can be set to minimize the size of the final
binary (usually at the cost of features or compile time).
See https://github.com/johnthagen/min-sized-rust for more information.

## License

ChirpStack Concentratord is distributed under the MIT license. See also
[LICENSE](https://github.com/brocaar/chirpstack-concentratord/blob/master/LICENSE).
