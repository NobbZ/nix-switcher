{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {flake-parts, ...} @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];

      perSystem = {
        inputs',
        self',
        pkgs,
        ...
      }: let
        inherit (inputs'.nixpkgs.legacyPackages.lib) pipe;

        pkgsWithOverlays = inputs'.nixpkgs.legacyPackages.lib.pipe inputs'.nixpkgs.legacyPackages [
          (pkgs: pkgs.extend inputs.rust-overlay.overlays.default)
        ];

        rustVersion = pipe "${inputs.self}/rust-toolchain.toml" [
          builtins.readFile
          builtins.fromTOML
          (toml: toml.toolchain.channel)
        ];

        rust = pkgs.rust-bin.stable.${rustVersion}.default;
        rustfmt = pkgs.rust-bin.nightly.latest.rustfmt;

        rustPlatform = pkgs.makeRustPlatform {
          rustc = rust;
          cargo = rust;
        };
      in {
        _module.args.pkgs = pkgsWithOverlays;

        formatter = pkgs.alejandra;

        packages.switcher = pkgs.callPackage ./nix/packages/switcher.nix {inherit rustPlatform inputs;};
        packages.default = self'.packages.switcher;

        devShells.default = pkgs.mkShell {
          packages = [rustfmt] ++ builtins.attrValues {
            inherit (pkgs) cargo-nextest cargo-audit cargo-deny cargo-tarpaulin rust-analyzer;
            inherit (pkgs) nil pkg-config openssl bacon;
            inherit rust;
          };
        };
      };
    };
}
