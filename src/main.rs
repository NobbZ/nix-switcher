#![warn(clippy::unwrap_used)]

use std::{path::Path, process::ExitStatus, str};

use clap::Parser;
use futures::future;
use tokio::{self, process::Command};
use tracing::{instrument, Level};
use tracing_futures::Instrument;
use tracing_subscriber::FmtSubscriber;

use eyre::{eyre, ContextCompat, Result, WrapErr};

use crate::{
    interface::{SwitcherCommand, SwitcherParser},
    provider::github,
};

mod cmd;
mod interface;
mod provider;

const OWNER: &str = "nobbz";
const REPO: &str = "nixos-config";

const BRANCH: &str = "main";

#[instrument]
async fn get_command_out(cmd: &mut Command) -> Result<String> {
    let out = cmd.output().await.wrap_err("running the command")?.stdout;

    Ok(str::from_utf8(&out)
        .wrap_err("converting the output to UTF-8")?
        .trim()
        .to_string())
}

#[instrument]
async fn spawn_command(cmd: &mut Command) -> Result<ExitStatus> {
    Ok(cmd.spawn().wrap_err("spawing the command")?.wait().await?)
}

#[instrument]
async fn retrieve_sha(owner: &str, repo: &str, branch: &str) -> Result<String> {
    github::get_latest_commit(owner, repo, Some(branch)).await
}

#[instrument]
async fn get_hostname() -> Result<String> {
    get_command_out(&mut Command::new("hostname"))
        .await
        .wrap_err("retrieving the hostname")
}

#[instrument]
async fn get_username() -> Result<String> {
    get_command_out(&mut Command::new("whoami"))
        .await
        .wrap_err("retrieving the current username")
}

#[instrument]
async fn get_tempfldr() -> Result<String> {
    get_command_out(Command::new("mktemp").arg("-d"))
        .await
        .wrap_err("creating the temporary folder")
}

#[instrument]
async fn check_nom() -> Result<Option<String>> {
    let location = get_command_out(Command::new("which").arg("nom"))
        .await
        .wrap_err("searching for `nom`")?;

    if location.is_empty() {
        return Ok(None);
    }

    Ok(Some(location))
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().wrap_err("installing 'color-eyre'")?;

    let args = <SwitcherParser as Parser>::parse();

    let max_level = match (&args.quiet, &args.verbose) {
        (true, 0) => Ok(Level::WARN),
        (false, 0) => Ok(Level::INFO),
        (false, 1) => Ok(Level::DEBUG),
        (false, 2) => Ok(Level::TRACE),
        (false, verb) => Err(eyre!("Verbosity max is 2, {} was requested", verb)),
        (true, _) => Err(eyre!("--quiet and --verbose are mutually exclusive")),
    }?;

    let builder = FmtSubscriber::builder().with_max_level(max_level);
    match &args.format {
        interface::LogFormat::Compact => builder.compact().init(),
        interface::LogFormat::Pretty => builder.pretty().init(),
        interface::LogFormat::Json => builder.json().init(),
    };

    if let SwitcherCommand::Complete(_) = args.command {
        cmd::complete::run(args).await?;
        return Ok(());
    }

    tracing::info!(
        "{name} v{version}",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION")
    );
    tracing::info!("Gathering info");

    let sha1_promise = retrieve_sha(OWNER, REPO, BRANCH);
    let host_promise = get_hostname();
    let user_promise = get_username();
    let temp_promise = get_tempfldr();
    let nom_promise = check_nom();

    let (sha1_res, host_res, user_res, temp_res, nom_res) = future::join5(
        sha1_promise,
        host_promise,
        user_promise,
        temp_promise,
        nom_promise,
    )
    .instrument(tracing::trace_span!("gather_info"))
    .await;

    let (sha1, host, user, temp, nom) = (sha1_res?, host_res?, user_res?, temp_res?, nom_res?);

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

    spawn_command(
        Command::new("nom").args([
            "build",
            "--keep-going",
            "-L",
            "--out-link",
            out_link
                .as_os_str()
                .to_str()
                .wrap_err_with(|| format!("converting {:?} to UTF-8", out_link))?,
            &nixos_config,
            &home_config,
        ]),
    )
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
