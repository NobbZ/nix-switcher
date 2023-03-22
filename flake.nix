{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    rust-overlay.url = "github:oxalica/rust-overlay";

    dream2nix.url = "github:nix-community/dream2nix";
    dream2nix.inputs.all-cabal-json.follows = "nixpkgs";
    dream2nix.inputs.nixpkgs.follows = "nixpkgs";
    dream2nix.inputs.flake-parts.follows = "flake-parts";
  };

  outputs = {flake-parts, ...} @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];

      imports = [inputs.dream2nix.flakeModuleBeta];

      perSystem = {
        system,
        config,
        pkgs,
        self',
        ...
      }: let
        pkgsWithOverlays = inputs.nixpkgs.legacyPackages.${system}.extend inputs.rust-overlay.overlays.default;
        rust = pkgs.rust-bin.fromRustupToolchainFile "${inputs.self}/rust-toolchain.toml";
      in {
        _module.args.pkgs = pkgsWithOverlays;

        formatter = pkgs.alejandra;

        dream2nix.inputs.self = {
          source = inputs.self;
          projects.switcher = {
            subsystem = "rust";
            translator = "cargo-lock";
          };
          packageOverrides.switcher-deps.add-openssl = {
            nativeBuildInputs = self: self ++ [pkgs.pkg-config];
            buildInputs = self: [pkgs.openssl];
          };

          packageOverrides.switcher.add-openssl = {
            nativeBuildInputs = self: self ++ [pkgs.pkg-config];
            buildInputs = self: [pkgs.openssl];
          };

          packageOverrides."^.*".set-toolchain = {
            cargo = rust;
            rustc = rust;
          };
        };

        packages.switcher = config.dream2nix.outputs.self.packages.switcher;
        packages.default = self'.packages.switcher;

        devShells.default = pkgs.mkShell {
          packages = builtins.attrValues {
            inherit (pkgs) rustfmt rust-analyzer cargo-nextest cargo-audit cargo-deny cargo-tarpaulin nil pkg-config openssl;
            inherit rust;
          };
        };
      };
    };
}
