mod cli;
mod config;
mod constants;
mod contract_bundle;
mod engine;
mod fuzzer;
mod generator;
mod output;
mod setup;
#[cfg(test)]
mod tests;
mod types;

use std::{
    fs::{
        create_dir_all,
        File,
    },
    io::{
        BufReader,
        BufWriter,
        Write,
    },
    path::PathBuf,
};

use crate::{
    config::Config,
    engine::FailedTrace,
};
use anyhow::{
    Ok,
    Result,
};
use clap::{
    self,
    Parser,
};
use cli::{
    Cli,
    Commands,
};
use engine::Engine;
use output::{
    ConsoleOutput,
    TuiOutput,
};
use setup::Setup;

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
            contracts,
        }) => {
            let setup = match contracts {
                Some(contracts) => Some(Setup::new(contracts)),
                None => None,
            };

            let config = match config {
                Some(config) => Config::from_yaml_file(config)?,
                None => Config::default(),
            };

            let contract_path = cli.contract;

            // Run the fuzzer
            let campaign_result = if config.use_tui || *tui {
                let mut engine = Engine::<TuiOutput>::new(contract_path, config, setup)?;
                engine.run_campaign()?
            } else {
                let mut engine =
                    Engine::<ConsoleOutput>::new(contract_path, config, setup)?;
                engine.run_campaign()?
            };

            // Create the results directory if it doesn't exist
            let results_dir = PathBuf::from("results");
            create_dir_all(&results_dir).expect("Failed to create results directory");

            // Determine the output file path
            let output_file_path = if let Some(output) = output {
                results_dir.join(output)
            } else {
                results_dir.join("failed_traces.json")
            };

            // Create and write to the output file
            if let std::result::Result::Ok(file) = File::create(&output_file_path) {
                let mut writer = BufWriter::new(file);

                serde_json::to_writer(&mut writer, &campaign_result.failed_traces)?;

                writer.flush()?;
            } else {
                eprintln!("Failed to create output file: {:?}", output_file_path);
            }
        }
        Some(Commands::Execute { input, config }) => {
            // Handle execute command
            println!("Executing contract: {:?}", cli.contract);
            println!("Using input: {:?}", input);

            // Read the input JSON file
            let file = File::open(input).expect("Failed to open input file");
            let reader = BufReader::new(file);

            // Deserialize the JSON data
            let failed_traces: Vec<FailedTrace> = serde_json::from_reader(reader)
                .expect("Failed to deserialize failed traces data");

            let config = match config {
                Some(config) => Config::from_yaml_file(config)?,
                None => Config::default(),
            };
            let contract_path = cli.contract;

            // Setup the engine
            let mut engine = Engine::<ConsoleOutput>::new(contract_path, config, None)?; // TODO: Maybe setup should also be used here?
            for (index, failed_trace) in failed_traces.iter().enumerate() {
                println!("Executing failed trace {}", index);
                engine.execute_failed_trace(failed_trace.to_owned());
                println!();
            }
        }
        None => {
            // Handle no subcommand
            println!("No subcommand provided. Use --help for more information.");
        }
    }

    Ok(())
}
