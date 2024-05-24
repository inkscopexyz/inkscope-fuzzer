use clap::{
    self,
    Parser,
    Subcommand,
};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input contract file
    #[arg(required = true)]
    pub contract: PathBuf,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Fuzz the contract
    Fuzz {
        /// Configuration yaml file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Enable TUI
        #[arg(short, long)]
        tui: bool,

        /// Dump failed traces to a json file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Execute the contract with the provided failed traces
    Execute {
        /// JSON file with failed traces
        #[arg(required = true)]
        input: PathBuf,

        // Configuration yaml file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}
