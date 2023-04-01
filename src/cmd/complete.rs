use std::io;
use std::{fs::File, io::Write};

use clap::CommandFactory;
use clap_complete::Shell;
use eyre::{anyhow, Result};
use tokio::task;
use tracing::instrument;

use crate::interface::{SwitcherCommand, SwitcherParser};

#[instrument]
pub async fn run(args: SwitcherParser) -> Result<()> {
    let SwitcherCommand::Complete(complete_args) = args.command else {
        return Err(anyhow!("shell completions called with wrong subcommand"));
    };

    let mut cmd = <SwitcherParser as CommandFactory>::command();
    let bin_name = cmd.get_name().to_string();

    task::spawn_blocking(move || -> Result<()> {
        let mut buf = match complete_args.file {
            None => Box::new(io::stdout()) as Box<dyn Write>,
            Some(p) => Box::new(File::create(p)?) as Box<dyn Write>,
        };

        clap_complete::generate(
            Into::<Shell>::into(complete_args.shell),
            &mut cmd,
            bin_name,
            &mut buf,
        );

        Ok(())
    })
    .await?
}
