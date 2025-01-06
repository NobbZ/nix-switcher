{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    rust-overlay.url = "github:oxalica/rust-overlay";

    cargo2nix.url = "github:cargo2nix/cargo2nix";
    cargo2nix.inputs.nixpkgs.follows = "nixpkgs";
    cargo2nix.inputs.rust-overlay.follows = "rust-overlay";
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
          (pkgs: pkgs.extend inputs.cargo2nix.overlays.default)
        ];

        rustVersion = pipe "${inputs.self}/rust-toolchain.toml" [
          builtins.readFile
          builtins.fromTOML
          (toml: toml.toolchain.channel)
        ];

        rust = pkgs.rust-bin.stable.${rustVersion}.default;

        rustPlatform = pkgs.makeRustPlatform {
          rustc = rust;
          cargo = rust;
        };
      in {
        _module.args.pkgs = pkgsWithOverlays;

        formatter = pkgs.alejandra;

        packages.switcher = rustPlatform.buildRustPackage {
          name = "switcher";
          version = "0.2.7-unstable-${inputs.self.rev or inputs.self.dirtyRev}";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [pkgs.pkg-config];
          buildInputs = [pkgs.openssl];
        };
        packages.default = self'.packages.switcher;

        devShells.default = pkgs.mkShell {
          packages = builtins.attrValues {
            inherit (pkgs) cargo-nextest cargo-audit cargo-deny cargo-tarpaulin rust-analyzer;
            inherit (pkgs) nil pkg-config openssl;
            inherit rust;
          };
        };
      };
    };
}
