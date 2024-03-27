use clap::{self, Parser};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Cli {
    /// input file
    #[clap(index = 1)]
    pub contract: PathBuf,

    #[arg(short, long, default_value = "config.yaml")]
    pub config: PathBuf,
}
