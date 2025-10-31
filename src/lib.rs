use std::process::ExitStatus;

use mockall_double::double;
use tokio::{self, process::Command};
use tracing::instrument;

use eyre::{Result, WrapErr};

use crate::provider::github;
#[double]
use crate::system::System;

mod provider;
pub mod system;

pub const OWNER: &str = "nobbz";
pub const REPO: &str = "nixos-config";
pub const BRANCH: &str = "main";

#[instrument]
async fn get_command_out(cmd: &mut Command) -> Result<String> {
    let system = System::default();

    system.get_command_out(cmd).await
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

#[instrument(skip(system))]
pub async fn format_nixos_config<S1, S2>(
    system: &System,
    flake_url: S1,
    hostname: S2,
) -> Option<String>
where
    S1: std::fmt::Display + std::fmt::Debug,
    S2: std::fmt::Display + std::fmt::Debug,
{
    if !system.is_nixos().await {
        return None;
    };

    Some(format!(
        "{}#nixosConfigurations.{}.config.system.build.toplevel",
        flake_url, hostname
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn regular_linux_doesnt_get_a_toplevel() {
        let mut mock_system = System::default();

        mock_system.expect_is_nixos().return_const(false);

        let result = format_nixos_config(&mock_system, "github:example/config", "nixos").await;

        assert_eq!(None, result);
    }

    #[tokio::test]
    async fn nixos_does_get_a_toplevel() {
        let mut mock_system = System::default();

        mock_system.expect_is_nixos().return_const(true);

        let result = format_nixos_config(&mock_system, "github:example/config", "nixos").await;

        assert_eq!(
            Some(
                "github:example/config#nixosConfigurations.nixos.config.system.build.toplevel"
                    .to_string()
            ),
            result
        );
    }
}
