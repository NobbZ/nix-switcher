use clap::{crate_authors, Parser};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(version, author = crate_authors!())]
pub struct SwParser {
    /// Generate shell completions
    #[clap(long)]
    pub completions: Option<Shell>,
}
