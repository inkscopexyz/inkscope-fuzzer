mod cli;
mod config;
mod constants;
mod contract_bundle;
mod engine;
mod fuzzer;
mod generator;
mod tui;
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
use engine::{CampaignData, Engine};

fn main() -> Result<()> {
    // Initialize the terminal UI
    let mut terminal = tui::init()?;
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

    let campaign_data = Arc::new(RwLock::new(CampaignData::new()));
    let mut app = tui::App {
        campaign_data: Arc::clone(&campaign_data),
        exit: false,
    };
    // Run the tui in a new thread
    std::thread::spawn(move || {
        app.run(&mut terminal).unwrap();
    });

    let mut engine = Engine::new(contract_path, config)?;
    let campaign_result = engine.run_campaign(&mut Some(Arc::clone(&campaign_data)))?;
    engine.print_campaign_result(&campaign_result);
    
    // Restore the terminal to its original state
    tui::restore()?;

    Ok(())
}
