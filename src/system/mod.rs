use std::{io::Error as IoError, path::Path, process::ExitStatus, str::Utf8Error};

use mockall::automock;
use thiserror::Error;
use tokio::process::Command;
use tracing::instrument;

#[derive(Default)]
pub struct System {}

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Can not retrieve the hostname")]
    HostnameError(#[source] Box<SystemError>),
    #[error("Can not run command `{:?}`", .0)]
    CommandError(String, #[source] IoError),
    #[error("Can not convert input to UTF-8: {:?}", .0)]
    StringConversionError(Vec<u8>, #[source] Utf8Error),
    #[error("Can not create temporary folder")]
    MkTempError(#[source] Box<SystemError>),
    #[error("Can not retrieve username")]
    UsernameError(#[source] Box<SystemError>),
}

#[automock]
impl System {
    /// Retrieves the current hosts hostname.
    ///
    /// # Errors
    ///
    /// Returns a `SystemError::HostnameError` if the hostname couldn't be
    /// retrieved.
    #[instrument(skip(self))]
    pub async fn get_hostname(&self) -> Result<String, SystemError> {
        self.get_command_out(&mut Command::new("hostname"))
            .await
            .map_err(|err| SystemError::HostnameError(Box::new(err)))
    }

    /// Retrieves the current users username.
    ///
    /// # Errors
    ///
    /// Returns a `SystemError::UsernameError` if the username couldn't be
    /// retrieved.
    #[instrument(skip(self))]
    pub async fn get_username(&self) -> Result<String, SystemError> {
        self.get_command_out(&mut Command::new("whoami"))
            .await
            .map_err(|err| SystemError::UsernameError(Box::new(err)))
    }

    /// Creates a temporary folder and returns its location in the file system as a `String`.
    ///
    /// # Errors
    ///
    /// Returns a `SystemError::MkTempError` if the temporary folder couldn't be
    /// created.
    #[instrument(skip(self))]
    pub async fn get_tempfldr(&self) -> Result<String, SystemError> {
        self.get_command_out(Command::new("mktemp").arg("-d"))
            .await
            .map_err(|err| SystemError::MkTempError(Box::new(err)))
    }

    /// Runs an arbitrary command and returns the output as a `String`.
    ///
    /// # Errors
    ///
    /// Returns one of the following errors:
    ///
    /// - `SystemError::CommandError` if there have been troubles running `cmd`
    /// - `SystemError::StringConversionError` if there have been troubles converting the output to
    ///    UTF-8
    #[instrument(skip(self))]
    pub async fn get_command_out(&self, cmd: &mut Command) -> Result<String, SystemError> {
        cmd.output()
            .await
            .map_err(|err| SystemError::CommandError(format!("{cmd:?}"), err))
            .map(|out| out.stdout)
            .and_then(|out| {
                std::str::from_utf8(&out)
                    .map_err(|err| SystemError::StringConversionError(out.clone(), err))
                    .map(|s| s.trim().into())
            })
    }

    /// Searches for a given program in the `PATH` variable.
    #[instrument(skip(self))]
    pub async fn which(&self, prg: &str) -> Result<Option<String>, SystemError> {
        self.get_command_out(Command::new("which").arg(prg))
            .await
            .map(|s| (!s.is_empty()).then_some(s))
    }

    /// Tries to find the `nom` executable
    pub async fn find_nom(&self) -> Result<Option<String>, SystemError> {
        self.which("nom").await
    }

    /// Tries to find the `gh` executable
    pub async fn find_gh(&self) -> Result<Option<String>, SystemError> {
        self.which("gh").await
    }

    #[instrument(skip(self))]
    pub async fn is_nixos(&self) -> bool {
        Path::new("/etc/NIXOS").exists()
    }

    pub async fn spawn_command(&self, cmd: &mut Command) -> Result<ExitStatus, SystemError> {
        cmd.spawn()
            .map_err(|err| SystemError::CommandError(format!("{cmd:?}"), err))?
            .wait()
            .await
            .map_err(|err| SystemError::CommandError(format!("{cmd:?}"), err))
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
