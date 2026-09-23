use std::path::PathBuf;

use clap::{ColorChoice, Parser};

/// Restore power. Learn a little Vim. Receive no additional compensation.
#[derive(Debug, Parser)]
#[command(name = "nightwatch", version, color = ColorChoice::Never)]
pub struct Args {
    /// Reproduce a facility using an unsigned 32-bit seed
    #[arg(long, conflicts_with = "load")]
    pub seed: Option<u32>,
    /// Restore a saved shift
    #[arg(long, conflicts_with = "practice")]
    pub load: Option<PathBuf>,
    /// Destination for explicit saves (EOF never saves)
    #[arg(long)]
    pub save: Option<PathBuf>,
    /// Skip induction and mix the console lesson order
    #[arg(long)]
    pub practice: bool,
    /// Use line-oriented commands without terminal escape codes
    #[arg(long)]
    pub plain: bool,
    /// Disable color styling
    #[arg(long)]
    pub no_color: bool,
}
