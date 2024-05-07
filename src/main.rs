mod cli;
mod config;
mod constants;
mod contract_bundle;
mod engine;
mod fuzzer;
mod generator;
mod info;
#[cfg(test)]
mod tests;
mod tui;
mod types;

use std::sync::{
    Arc,
    RwLock,
};

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
use engine::{
    CampaignData,
    CampaignStatus,
    Engine,
};
use info::{
    ConsoleOutput,
    TuiOutput,
};

fn main() -> Result<()> {
    // This initializes the logging. The code uses debug! info! trace! and error! macros
    // You can enable the output via the environment variable RUST_LOG
    env_logger::init();

    // Parse the command line arguments
    let cli = Cli::parse();

    // Used for developement when the Config format is changed
    // Config::default().to_yaml_file(&cli.config)?;
    let config = match cli.config {
        Some(config) => Config::from_yaml_file(config)?,
        None => Config::default(),
    };
    let contract_path = cli.contract;

    let campaign_data = Arc::new(RwLock::new(CampaignData::default()));

    // Run the fuzzer
    if config.use_tui {
        let mut engine = Engine::<TuiOutput>::new(contract_path, config)?;
        engine.run_campaign(&mut Arc::clone(&campaign_data))?;
    } else {
        let mut engine = Engine::<ConsoleOutput>::new(contract_path, config)?;
        engine.run_campaign(&mut Arc::clone(&campaign_data))?;
    }

    Ok(())
}
