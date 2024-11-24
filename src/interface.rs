// Dont't use crate::
// We are getting called by build.rs

use clap::ArgAction::Count;
use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell as ClapShell;

use crate::flake::r#ref::FlakeRef;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
// #[command(propagate_version = True)]
/// Yet another nix deployment tool.
pub struct SwitcherParser {
    #[arg(short, long, action = Count, global = true)]
    /// Increase verbosity. (up tp 2 times)
    pub verbose: u8,

    #[arg(short, long, global = true)]
    /// Omit any output except warnings and errors from switcher
    pub quiet: bool,

    #[arg(value_enum, short, long, global = true, default_value_t = LogFormat::Compact)]
    /// Sets the log format
    pub format: LogFormat,

    #[command(subcommand)]
    pub command: SwitcherCommand,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogFormat {
    Compact,
    Pretty,
    Json,
}

#[derive(Debug, Subcommand, Clone)]
pub enum SwitcherCommand {
    Build(BuildArgs),
    Complete(CompleteArgs),
}

#[derive(Debug, Args, Clone)]
/// Builds a system and switch to its configuration
pub struct BuildArgs {
    #[arg(short = 'F', long)]
    /// which flake to use (no fragment allowed)
    pub flake: FlakeRef,

    #[arg(short = 'H', long)]
    /// Host to build (will default to the current system)
    pub host: Option<String>,

    #[arg(short = 'U', long)]
    /// User(s) to build (will default to all found for the host)
    pub user: Vec<String>,

    #[arg(long, default_value_t = false)]
    /// Ignore user(s) and build the host only
    pub only_system: bool,

    #[arg(long, default_value_t = false)]
    /// try to find all the hosts and build them
    pub all_systems: bool,
}

#[derive(Debug, Args, Clone)]
/// Generate a completion for the requested shell
pub struct CompleteArgs {
    #[arg()]
    /// The shell to generate completions for
    pub shell: Shell,

    #[arg(short = 'F', long)]
    /// File to write to, if missing `stdout` is used
    pub file: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl From<Shell> for ClapShell {
    fn from(val: Shell) -> Self {
        match val {
            Shell::Bash => ClapShell::Bash,
            Shell::Elvish => ClapShell::Elvish,
            Shell::Fish => ClapShell::Fish,
            Shell::PowerShell => ClapShell::PowerShell,
            Shell::Zsh => ClapShell::Zsh,
        }
    }
}
