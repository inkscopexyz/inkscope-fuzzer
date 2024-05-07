mod cli;
mod config;
mod constants;
mod contract_bundle;
mod engine;
mod fuzzer;
mod generator;
mod tui;
mod info;
#[cfg(test)]
mod tests;
mod types;

use std::sync::{Arc, RwLock};

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
use engine::{CampaignData, CampaignStatus, Engine};
use info::Info;

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
    
    // Init info module
    let mut info_mod = Info::new();
    info_mod.init(Arc::clone(&campaign_data), true )?;

    // Run the fuzzer
    let mut engine = Engine::new(contract_path, config)?;
    let campaign_result = engine.run_campaign(&mut Arc::clone(&campaign_data))?;
    
    // Mark the campaign as finished
    campaign_data.write().unwrap().status = CampaignStatus::Finished;
    
    // Finalize the info module
    info_mod.finalize()?;

    // Print the campaign result
    engine.print_campaign_result(&campaign_result);

    Ok(())
}
