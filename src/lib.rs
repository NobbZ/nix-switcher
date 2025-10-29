use std::{path::Path, process::ExitStatus, str};

use tokio::{self, process::Command};
use tracing::instrument;

use eyre::{Result, WrapErr};

use crate::provider::github;

mod provider;

pub const OWNER: &str = "nobbz";
pub const REPO: &str = "nixos-config";
pub const BRANCH: &str = "main";

#[instrument]
async fn get_command_out(cmd: &mut Command) -> Result<String> {
    let out = cmd.output().await.wrap_err("running the command")?.stdout;

    Ok(str::from_utf8(&out)
        .wrap_err("converting the output to UTF-8")?
        .trim()
        .to_string())
}

#[instrument]
pub async fn spawn_command(cmd: &mut Command) -> Result<ExitStatus> {
    Ok(cmd.spawn().wrap_err("spawing the command")?.wait().await?)
}

#[instrument]
pub async fn retrieve_sha(owner: &str, repo: &str, branch: &str) -> Result<String> {
    github::get_latest_commit(owner, repo, Some(branch)).await
}

#[instrument]
pub async fn get_hostname() -> Result<String> {
    get_command_out(&mut Command::new("hostname"))
        .await
        .wrap_err("retrieving the hostname")
}

#[instrument]
pub async fn get_username() -> Result<String> {
    get_command_out(&mut Command::new("whoami"))
        .await
        .wrap_err("retrieving the current username")
}

#[instrument]
pub async fn get_tempfldr() -> Result<String> {
    get_command_out(Command::new("mktemp").arg("-d"))
        .await
        .wrap_err("creating the temporary folder")
}

#[instrument]
pub async fn check_nom() -> Result<Option<String>> {
    let location = get_command_out(Command::new("which").arg("nom"))
        .await
        .wrap_err("searching for `nom`")?;

    if location.is_empty() {
        return Ok(None);
    }

    Ok(Some(location))
}

#[instrument]
pub async fn check_gh() -> Result<Option<String>> {
    let location = get_command_out(Command::new("which").arg("gh"))
        .await
        .wrap_err("searching for `gh`")?;

    if location.is_empty() {
        return Ok(None);
    }

    Ok(Some(location))
}

#[instrument]
async fn is_nixos() -> bool {
    Path::new("/etc/NIXOS").exists()
}

#[instrument]
pub async fn format_nixos_config<S1, S2>(flake_url: S1, hostname: S2) -> Option<String>
where
    S1: std::fmt::Display + std::fmt::Debug,
    S2: std::fmt::Display + std::fmt::Debug,
{
    if !is_nixos().await {
        return None;
    };

    Some(format!(
        "{}#nixosConfigurations.{}.config.system.build.toplevel",
        flake_url, hostname
    ))
}
