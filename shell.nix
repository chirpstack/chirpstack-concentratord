{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-25.05.tar.gz") {} }:

pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = [
    pkgs.cacert
    pkgs.rustup
    # cargo-cross can be used once version > 0.2.5, as 0.2.5 does not work well
    # with nightly toolchain. It is for now installed through make dev-dependencies.
    # pkgs.cargo-cross
    pkgs.jq
    pkgs.opkg-utils
  ];
  shellHook = ''
    export PATH=$PATH:~/.cargo/bin
  '';
  DOCKER_BUILDKIT = "1";
  NIX_STORE = "/nix/store";
}
