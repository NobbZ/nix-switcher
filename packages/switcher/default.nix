{
  lib,
  rustPlatform,
  makeWrapper,
  gh,
  hostname,
  coreutils,
  nix,
  nixos-rebuild,
  home-manager,
  ncurses6,
}: let
  runtimeDeps = [gh hostname coreutils nix nixos-rebuild home-manager ncurses6];
in
  rustPlatform.buildRustPackage {
    pname = "nobbz-switcher";
    version = "0.1.0";

    nativeBuildInputs = [makeWrapper];

    src = ./.;

    cargoSha256 = "sha256-KROqjfumSwyj1MSeq8fvc3k4twIKor8XSPhg3o5MjS8=";

    postInstall = ''
      wrapProgram $out/bin/switcher \
        --set PATH "${lib.makeBinPath runtimeDeps}:/run/wrappers/bin"
    '';
  }
