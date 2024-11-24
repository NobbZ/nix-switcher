// Dont't use crate::
// We are getting called by build.rs

use clap::ArgAction::Count;
use clap::{Args, Parser, Subcommand, ValueEnum};

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

#[derive(Debug, Subcommand)]
pub enum SwitcherCommand {
    Build(BuildArgs),
}

#[derive(Debug, Args)]
/// Builds a system and
pub struct BuildArgs {
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
