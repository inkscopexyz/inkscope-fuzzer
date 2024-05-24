mod cli;
mod config;
mod constants;
mod contract_bundle;
mod engine;
mod fuzzer;
mod generator;
mod output;
#[cfg(test)]
mod tests;
mod types;

use std::{
    fs::File,
    io::{
        BufWriter,
        Write,
    },
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
use cli::{Cli, Commands};
use engine::Engine;
use output::{
    ConsoleOutput,
    TuiOutput,
};

fn main() -> Result<()> {
    // This initializes the logging. The code uses debug! info! trace! and error! macros
    // You can enable the output via the environment variable RUST_LOG
    env_logger::init();

    // Parse the command line arguments
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Fuzz {
            config,
            tui,
            output,
        }) => {
            let config = match config {
                Some(config) => Config::from_yaml_file(config)?,
                None => Config::default(),
            };
            let contract_path = cli.contract;

            // Run the fuzzer
            let campaign_result = if config.use_tui || *tui {
                let mut engine = Engine::<TuiOutput>::new(contract_path, config)?;
                engine.run_campaign()?
            } else {
                let mut engine = Engine::<ConsoleOutput>::new(contract_path, config)?;
                engine.run_campaign()?
            };

            if let Some(output) = output {
                let file = File::create(output)?;
                let mut writer = BufWriter::new(file);
                for ft in campaign_result.failed_traces {
                    serde_json::to_writer(&mut writer, &ft)?;
                }
                writer.flush()?;
            }
        }
        Some(Commands::Execute { input }) => {
            // Handle execute command
            println!("Executing contract: {:?}", cli.contract);
            println!("Using input: {:?}", input);
        }
        None => {
            // Handle no subcommand
            println!("No subcommand provided. Use --help for more information.");
        }
    }

    Ok(())
}
