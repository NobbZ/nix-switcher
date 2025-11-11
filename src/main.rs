#![warn(clippy::unwrap_used)]

use core::panic;
use std::path::Path;

use futures::future;
use microxdg::XdgApp;
use tokio::{self, process::Command};
use tracing::Instrument;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use eyre::{ContextCompat, Result, WrapErr};

use switcher::config::Config;
use switcher::system::System;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().wrap_err("installing 'color-eyre'")?;
    FmtSubscriber::builder().with_max_level(Level::DEBUG).init();

    tracing::info!("Gathering info");

    let system = System::default();
    let xdg_app = XdgApp::new("switcher")?;

    let conf: Config = Config::figment(&xdg_app)?.extract()?;
    let rc = conf.repo;

    let sha1_promise =
        switcher::provider::github::get_latest_commit(&rc.owner, &rc.repo, rc.branch);
    let host_promise = system.get_hostname();
    let user_promise = system.get_username();
    let temp_promise = system.get_tempfldr();
    let nom_promise = switcher::check_nom();
    let gh_promise = switcher::check_gh();

    let (nom_res, gh_res) = future::join(nom_promise, gh_promise)
        .instrument(tracing::trace_span!("checking dependencies"))
        .await;

    let (sha1_res, host_res, user_res, temp_res) =
        future::join4(sha1_promise, host_promise, user_promise, temp_promise)
            .instrument(tracing::trace_span!("gather_info"))
            .await;

    let (sha1, host, user, temp, nom, gh) = (
        sha1_res?, host_res?, user_res?, temp_res?, nom_res?, gh_res?,
    );

    tracing::info!(%sha1, %host, %user, %temp, ?nom, ?gh, "Gathered info");

    match (gh.is_none(), nom.is_none()) {
        (true, true) => panic!("GH-CLI and Nix Output Monitor were not found"),
        (true, _) => panic!("GH-CLI was not found"),
        (_, true) => panic!("Nix Output Monitor was not found"),
        _ => (),
    }

    tracing::info!("Building strings");

    let flake_url = format!("github:{}/{}?ref={}", rc.owner, rc.repo, sha1);
    let nixos_config = switcher::format_nixos_config(&system, &flake_url, &host).await;
    let nixos_rebuild = format!("{flake_url}#{host}");
    let home_config = format!("{flake_url}#homeConfigurations.{user}@{host}.activationPackage");
    let home_manager = format!("{flake_url}#{user}@{host}");
    let out_link = Path::new(&temp).join("result");

    tracing::info!(%flake_url, ?nixos_config, %nixos_rebuild, %home_config, %home_manager, ?out_link, "Built strings");
    tracing::info!("Starting to build");

    switcher::spawn_command(
        Command::new("nom")
            .args([
                "build",
                "--keep-going",
                "-L",
                "--out-link",
                out_link
                    .as_os_str()
                    .to_str()
                    .wrap_err_with(|| format!("converting {out_link:?} to UTF-8"))?,
                &home_config,
            ])
            .args(nixos_config.as_slice()),
    )
    .instrument(tracing::debug_span!("nom_build"))
    .await?;

    tracing::info!("Finished building");
    if nixos_config.is_some() {
        tracing::info!(%host, "Switching system configuration");

        switcher::spawn_command(Command::new("nixos-rebuild").args([
            "switch",
            "--use-remote-sudo",
            "--flake",
            &nixos_rebuild,
        ]))
        .instrument(tracing::debug_span!("nixos_switch"))
        .await?;

        tracing::info!(%host, "Switched system configuration");
    } else {
        tracing::info!(%host, "Not a NixOS, skipping activation");
    }
    tracing::info!(%user, %host, "Switching user configuration");

    switcher::spawn_command(Command::new("home-manager").args([
        "switch",
        "--flake",
        &home_manager,
    ]))
    .instrument(tracing::debug_span!("home_switch"))
    .await?;

    tracing::info!(%user, %host, "Switched user configuration");
    tracing::info!(%temp, "Cleaning up");

    switcher::spawn_command(Command::new("rm").args(["-rfv", &temp])).await?;

    Ok(())
}
