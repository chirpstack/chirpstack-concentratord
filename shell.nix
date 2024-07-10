{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-24.05.tar.gz") {} }:

pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = [
    pkgs.cacert
    pkgs.rustup
    pkgs.cargo-cross
    pkgs.jq
    pkgs.opkg-utils
  ];
  DOCKER_BUILDKIT = "1";
  NIX_STORE = "/nix/store";
}
