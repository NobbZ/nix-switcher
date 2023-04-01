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
        lib,
        ...
      }: let
        pkgsWithOverlays = inputs.nixpkgs.legacyPackages.${system}.extend inputs.rust-overlay.overlays.default;
        rust = pkgs.rust-bin.fromRustupToolchainFile "${inputs.self}/rust-toolchain.toml";
        rust-min = pkgs.rust-bin.fromRustupToolchainFile "${inputs.self}/rust-toolchain-min.toml";

        withCompletions = switcher: shells: let
          completionBuilder = shell:
            pkgs.runCommand "${switcher.pname}-${shell}-${switcher.version}" {nativeBuildInputs = [pkgs.installShellFiles];} ''
              ${lib.getExe switcher} complete ${shell} --file switcher.${shell}
              installShellCompletion switcher.${shell}
            '';
          completions = builtins.map completionBuilder shells;
          name = "${switcher.pname}-${switcher.version}";
        in
          pkgs.symlinkJoin {
            inherit name;

            paths = [switcher] ++ completions;
          };
      in {
        _module.args.pkgs = pkgsWithOverlays;

        formatter = pkgs.alejandra;

        dream2nix.inputs = let
          commonData = {
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
              overrideAttrs = self:
                self
                // {
                  meta =
                    (self.meta or {})
                    // {
                      mainProgram = "switcher";
                    };
                };
            };
          };

          makeInput = name: rust: {
            "${name}" = lib.mkMerge [
              commonData
              {
                packageOverrides."^.*".set-toolchain = {
                  cargo = rust;
                  rustc = rust;
                };
              }
            ];
          };
        in
          lib.mkMerge [
            (makeInput "self" rust)
            (makeInput "minimal-rust" rust-min)
          ];

        packages.switcher-no-completion = config.dream2nix.outputs.self.packages.switcher;
        packages.switcher = lib.makeOverridable ({shells}: withCompletions config.packages.switcher-no-completion shells) {shells = ["bash" "fish" "zsh"];};
        packages.default = config.packages.switcher;

        checks.switcher = config.packages.switcher;
        checks.minimal = config.dream2nix.outputs.minimal-rust.packages.switcher;

        devShells.default = pkgs.mkShell {
          packages = builtins.attrValues {
            inherit (pkgs) cargo-nextest cargo-audit cargo-deny cargo-tarpaulin nil pkg-config openssl;
            inherit rust;
          };
        };
      };
    };
}
