# AGENTS.md

Rust CLI tool (`switcher`) that applies the author's NixOS/Home Manager configs: fetches the latest commit SHA from a GitHub config repo via GraphQL, builds NixOS/Home Manager flakes with `nom`, then switches.

## VCS / contribution model

- **Primary VCS is Jujutsu (`jj`)**, not plain git. The working copy is jj-managed (detached HEAD). Use `jj status`/`jj log` to inspect state; `git` commands may behave unexpectedly here.
- Code review is via **Gerrit** (gerrithub.io), not GitHub PRs. No GitHub Actions CI exists.
- Do **not** push with raw `git` when `jj` is available. Upload reviewable changes with jj, e.g. `jj gerrit upload --remote-branch main` (use the target branch that's sensible for the change). Only fall back to `git` if `jj` is unavailable.
- Keep changes small and in a single commit; iterate with `--amend` when pushing to Gerrit.

## Developer commands

Rust toolchain is **pinned to 1.83.0** in `rust-toolchain.toml` (with rustfmt, clippy, rust-analyzer).

- Enter dev shell (tools like `cargo-nextest`, `cargo-audit`, `cargo-deny`, `cargo-tarpaulin`, `alejandra` are only here): `nix develop` or `direnv allow`
- Test: `cargo test` (or `cargo nextest run` in dev shell)
- Lint: `cargo clippy`
- Format: `cargo fmt` (project uses **nightly rustfmt** configured in `.rustfmt.toml`; `cargo fmt` in flake derives nightly rustfmt)
- Nix format: `alejandra` (the flake formatter)
- Build package: `nix build .#switcher`
- Package derivation: `nix/packages/switcher.nix` (uses `cargoLock.lockFile`; keep `Cargo.lock` updated when changing dependencies). The legacy `default.nix` is **stale** (version 0.2.3, `cargoSha256`); prefer the flake.

## Architecture notes

- Binary entrypoint `src/main.rs`; modules live in `src/lib.rs` (`config`, `interface`, `provider`, `system`).
- `system::System` is the abstraction over external commands (hostname, username, tempdir, process spawning). It is **mocked with `mockall`/`mockall_double`** in tests — when adding methods that shell out, keep them on `System` so tests can mock them.
- Provider layer (`provider/`) currently only has GitHub (GraphQL via `graphql_client`). Auth uses `gh auth token`.
- Config loads via `figment` from XDG dirs (TOML/YAML/JSON) and `SWITCHER_*` env vars.
- Handles both NixOS (`nixos-rebuild switch`) and Home Manager (`home-manager switch`) paths.

## Gotchas

- `flake.nix` reads `rust-toolchain.toml` for the toolchain version; the dev shell adds nightly rustfmt even though the normal toolchain is stable.
- Rust code targets Linux/macOS (aarch64-darwin is a flake system), so be mindful of platform-specific command output parsing (`system` module has macOS `hostname` handling).
- The two primary config files to keep consistent when releasing: `Cargo.toml` version and `nix/packages/switcher.nix`.
