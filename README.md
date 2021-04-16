# ChirpStack Concentratord

![Tests](https://github.com/brocaar/chirpstack-concentratord/actions/workflows/main.yml/badge.svg?branch=master)

ChirpStack Concentratord is an open-source LoRa(WAN) concentrator daemon, part
of the [ChirpStack](https://www.chirpstack.io/) project. It exposes a [ZeroMQ](https://zeromq.org/)
based API that can be used by one or multiple applications to interact with
gateway hardware. By implementing and abstracting the the hardware specifics
in a separate daemon and exposing this over a ZeroMQ based API, the packet
forwarding application can be completely decoupled from the gateway hardware.
It also allows for running multiple packet forwarding applications simultaniously.

## Architecture

![architecture](https://www.chirpstack.io/static/img/graphs/architecture.dot.png)

## Links

* [Documentation](https://www.chirpstack.io/concentratord/)

## License

ChirpStack Concentratord is distributed under the MIT license. See
[LICENSE](https://github.com/brocaar/chirpstack-concentratord/blob/master/LICENSE).
