{
  rustPlatform,
  inputs,
  lib,
  pkg-config,
  openssl,
}: let
  fs = lib.fileset;
in
  rustPlatform.buildRustPackage {
    name = "switcher";
    version = "0.2.7-unstable-${inputs.self.rev or inputs.self.dirtyRev}";

    src = fs.toSource {
      root = ../..;
      fileset = fs.difference ./../.. (fs.unions [./../../flake.lock ./../../flake.nix ./../../nix]);
    };

    cargoLock.lockFile = ./../../Cargo.lock;

    nativeBuildInputs = [pkg-config];
    buildInputs = [openssl];
  }
