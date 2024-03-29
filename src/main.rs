use std::{error::Error, io::Error as IoError, path::Path, process::ExitStatus, str};

use futures::future;
use tokio::{self, process::Command};
use tracing::{instrument, Level};
use tracing_futures::Instrument;
use tracing_subscriber::FmtSubscriber;

use crate::provider::github;

mod provider;

const OWNER: &str = "nobbz";
const REPO: &str = "nixos-config";

const BRANCH: &str = "main";

#[instrument]
async fn get_command_out(cmd: &mut Command) -> String {
    let out = cmd.output().await.unwrap().stdout;

    str::from_utf8(&out).unwrap().trim().to_string()
}

#[instrument]
async fn spawn_command(cmd: &mut Command) -> Result<ExitStatus, IoError> {
    cmd.spawn().unwrap().wait().await
}

#[instrument]
async fn retrieve_sha(owner: &str, repo: &str, branch: &str) -> String {
    github::get_latest_commit(owner, repo, Some(branch))
        .await
        .unwrap()
}

#[instrument]
async fn get_hostname() -> String {
    get_command_out(&mut Command::new("hostname")).await
}

#[instrument]
async fn get_username() -> String {
    get_command_out(&mut Command::new("whoami")).await
}

#[instrument]
async fn get_tempfldr() -> String {
    get_command_out(Command::new("mktemp").arg("-d")).await
}

#[instrument]
async fn check_nom() -> Option<String> {
    let location = get_command_out(Command::new("which").arg("nom")).await;

    if location.is_empty() {
        return None;
    }

    Some(location)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    FmtSubscriber::builder().with_max_level(Level::DEBUG).init();

    tracing::info!("Gathering info");

    let sha1_promise = retrieve_sha(OWNER, REPO, BRANCH);
    let host_promise = get_hostname();
    let user_promise = get_username();
    let temp_promise = get_tempfldr();
    let nom_promise = check_nom();

    let (sha1, host, user, temp, nom) = future::join5(
        sha1_promise,
        host_promise,
        user_promise,
        temp_promise,
        nom_promise,
    )
    .instrument(tracing::trace_span!("gather_info"))
    .await;

    tracing::info!(%sha1, %host, %user, %temp, ?nom, "Gathered info");

    if nom.is_none() {
        panic!("Nix output monitor not found");
    }

    tracing::info!("Building strings");

    let flake_url = format!("github:{}/{}?ref={}", OWNER, REPO, sha1);
    let nixos_config = format!(
        "{}#nixosConfigurations.{}.config.system.build.toplevel",
        flake_url, host
    );
    let nixos_rebuild = format!("{}#{}", flake_url, host);
    let home_config = format!(
        "{}#homeConfigurations.{}@{}.activationPackage",
        flake_url, user, host
    );
    let home_manager = format!("{}#{}@{}", flake_url, user, host);
    let out_link = Path::new(&temp).join("result");

    tracing::info!(%flake_url, %nixos_config, %nixos_rebuild, %home_config, %home_manager, ?out_link, "Built strings");
    tracing::info!("Starting to build");

    spawn_command(Command::new("nom").args([
        "build",
        "--keep-going",
        "-L",
        "--out-link",
        out_link.as_os_str().to_str().unwrap(),
        &nixos_config,
        &home_config,
    ]))
    .instrument(tracing::debug_span!("nom_build"))
    .await?;

    tracing::info!("Finished building");
    tracing::info!(%host, "Switching system configuration");

    spawn_command(Command::new("nixos-rebuild").args([
        "switch",
        "--use-remote-sudo",
        "--flake",
        &nixos_rebuild,
    ]))
    .instrument(tracing::debug_span!("nixos_switch"))
    .await?;

    tracing::info!(%host, "Switched system configuration");
    tracing::info!(%user, %host, "Switching user configuration");

    spawn_command(Command::new("home-manager").args(["switch", "--flake", &home_manager]))
        .instrument(tracing::debug_span!("home_switch"))
        .await?;

    tracing::info!(%user, %host, "Switched user configuration");
    tracing::info!(%temp, "Cleaning up");

    spawn_command(Command::new("rm").args(["-rfv", &temp])).await?;

    Ok(())
}
