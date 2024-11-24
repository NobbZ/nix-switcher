#![warn(clippy::unwrap_used)]

use std::process::ExitStatus;

use clap::Parser;
use tokio::{self, process::Command};
use tracing::{instrument, Level};
use tracing_subscriber::FmtSubscriber;

use eyre::{eyre, Result, WrapErr};

use crate::{
    interface::{SwitcherCommand, SwitcherParser},
    provider::github,
};

mod cmd;
mod flake;
mod interface;
mod provider;

// const OWNER: &str = "nobbz";
// const REPO: &str = "nixos-config";
//
// const BRANCH: &str = "main";

#[instrument]
async fn get_command_out(cmd: &mut Command) -> Result<String> {
    let out = cmd.output().await.wrap_err("running the command")?.stdout;

    Ok(std::str::from_utf8(&out)
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

    tracing::info!(
        "{name} v{version}",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION")
    );

    match args.command {
        SwitcherCommand::Complete(_) => cmd::complete::run(args).await?,
        SwitcherCommand::Build(_) => cmd::build::run(args).await?,
    };

    Ok(())
}
