use std::path::Path;

use eyre::{Context, Result};
use mockall::automock;
use tokio::process::Command;
use tracing::instrument;

#[derive(Default)]
pub struct System {}

#[automock]
impl System {
    #[instrument(skip(self))]
    pub async fn get_hostname(&self) -> Result<String> {
        self.get_command_out(&mut Command::new("hostname"))
            .await
            .wrap_err("retrieving the hostname")
    }

    #[instrument(skip(self))]
    pub async fn get_username(&self) -> Result<String> {
        self.get_command_out(&mut Command::new("whoami"))
            .await
            .wrap_err("retrieving the current username")
    }

    #[instrument(skip(self))]
    pub async fn get_tempfldr(&self) -> Result<String> {
        self.get_command_out(Command::new("mktemp").arg("-d"))
            .await
            .wrap_err("creating the temporary folder")
    }

    #[instrument(skip(self))]
    pub async fn get_command_out(&self, cmd: &mut Command) -> Result<String> {
        let out = cmd.output().await.wrap_err("running the command")?.stdout;

        Ok(std::str::from_utf8(&out)
            .wrap_err("converting the output to UTF-8")?
            .trim()
            .to_string())
    }

    #[instrument(skip(self))]
    pub async fn is_nixos(&self) -> bool {
        Path::new("/etc/NIXOS").exists()
    }
}

#[cfg(test)]
mod tests {
    use mockall_double::double;

    #[double]
    use super::System;

    #[tokio::test]
    async fn this_test_only_verifies_that_i_semi_understood_mockall() {
        let mut mock = System::default();

        mock.expect_get_hostname()
            .returning(|| Ok("nixos".to_string()));

        assert_eq!("nixos", mock.get_hostname().await.unwrap());
    }
}
