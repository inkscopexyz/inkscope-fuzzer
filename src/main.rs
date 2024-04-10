mod cli;
mod config;
mod constants;
mod engine;
mod fuzzer;
mod generator;
#[cfg(test)]
mod tests;
mod types;
mod contract_bundle;

use crate::config::Config;
use anyhow::{
    Ok,
    Result,
};
use clap::{
    self,
    Parser,
};
use cli::Cli;
use engine::Engine;

fn main() -> Result<()> {
    // This initializes the logging. The code uses debug! info! trace! and error! macros
    // You can enable the output via the environment variable RUST_LOG
    env_logger::init();

    // Parse the command line arguments
    let cli = Cli::parse();

    // Used for developement when the Config format is changed
    // Config::default().to_yaml_file(&cli.config)?;
    let config = Config::from_yaml_file(&cli.config).expect("failed to parse yaml file");
    let contract_path = cli.contract;

    let mut engine = Engine::new(contract_path, config)?;
    let campaign_result = engine.run_campaign()?;
    engine.print_campaign_result(&campaign_result);

    Ok(())
}
