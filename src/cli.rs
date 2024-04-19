use clap::{
    self,
    Parser,
};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Cli {
    /// input file
    #[clap(index = 1)]
    pub contract: PathBuf,

    #[arg(short, long)]
    pub config: Option<PathBuf>,
}
