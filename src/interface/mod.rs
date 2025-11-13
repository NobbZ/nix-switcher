use clap::{crate_authors, Args, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(version, author = crate_authors!())]
pub struct SwParser {
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}

#[derive(Debug, Subcommand)]
#[command()]
pub enum SubCommand {
    /// Generate shell completions
    Completions(Completions),
}

#[derive(Debug, Args)]
#[command()]
pub struct Completions {
    /// Shell to generate completions for
    #[clap()]
    pub shell: Shell,

    /// filename to write the completions to (- for stdout)
    #[clap(short, long, default_value = "-")]
    pub file: String,
}
