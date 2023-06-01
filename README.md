# ChirpStack Concentratord

![Tests](https://github.com/brocaar/chirpstack-concentratord/actions/workflows/main.yml/badge.svg?branch=master)

ChirpStack Concentratord is an open-source LoRa(WAN) concentrator daemon, part
of the [ChirpStack](https://www.chirpstack.io/) project. It exposes a [ZeroMQ](https://zeromq.org/)
based API that can be used by one or multiple applications to interact with
gateway hardware. By implementing and abstracting the the hardware specifics
in a separate daemon and exposing this over a ZeroMQ based API, the packet
forwarding application can be completely decoupled from the gateway hardware.
It also allows for running multiple packet forwarding applications simultaniously.

## Documentation and binaries

Please refer to the [ChirpStack](https://www.chirpstack.io/) website for
documentation and pre-compiled binaries.

## Building from source

### Requirements

Building ChirpStack Concentratord requires:

* [Nix](https://nixos.org/download.html) (recommended) and
* [Docker](https://www.docker.com/)

#### Nix

Nix is used for setting up the development environment which is used for local
development and for creating the binaries.

If you do not have Nix installed and do not wish to install it, you could
install the packages listed in `shell.nix` by hand, using your package-manager of
choice.

#### Docker

Docker is used by [cross-rs](https://github.com/cross-rs/cross) for cross-compiling.

### Starting the development shell

Run the following command to start the development shell:

```bash
nix-shell
```

### Running tests

Execute the following command to run the tests:

```bash
make test
```

### Building binaries

Execute the following commands to build the ChirpStack Concentratord binaries
and packages:

```bash
# Only build binaries
make build

# Build binaries + distributable packages.
make dist
```

## License

ChirpStack Concentratord is distributed under the MIT license. See
[LICENSE](https://github.com/brocaar/chirpstack-concentratord/blob/master/LICENSE).
