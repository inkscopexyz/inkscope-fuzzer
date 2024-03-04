mod arguments;
mod config;
mod constants;
mod fuzzer;
mod types;
mod engine;


use crate::config::Config;
use std::path::PathBuf;
use clap::{self, Parser};
use anyhow::{anyhow, Ok, Result};
use engine::Engine;



#[derive(Debug, Parser)]
struct Cli {
    /// input file
    #[clap(index = 1)]
    pub contract: PathBuf,

    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    // This initializes the logging. The code uses debug! info! trace! and error! macros
    // You can enable the output via the environment variable RUST_LOG
    env_logger::init();

    // Parse the command line arguments
    let cli = Cli::parse();

    // Used for developement when the Config format is changed
    //Config::default().to_yaml_file(&cli.config)?;
    let config = Config::from_yaml_file(&cli.config).expect("failed to parse yaml file");
    let contract_path = cli.contract;

    let mut engine = Engine::new(contract_path, config)?;
    engine.run_campaign(1000)?;

    Ok(())
}

// fn mainy() -> Result<()> {
//     // This initializes the logging. The code uses debug! info! trace! and error! macros
//     // You can enable the output via the environment variable RUST_LOG
//     env_logger::init();
//     let mut fuzzer = RuntimeFuzzer::new(
//         PathBuf::from("./test-contracts/ityfuzz/target/ink/ityfuzz.contract"),
//         Constants::default(),
//     )?;

//     let mut session: Session<MinimalRuntime> = Session::<MinimalRuntime>::new()?;

//     let start_time = std::time::Instant::now();
//     for _ in 0..1000 {
//         let r = fuzzer.run(&mut session);
//         println!("Result: {:?}", r);
//     }
//     println!("Elapsed time: {:?}", start_time.elapsed());
//     Ok(())
// }

// fn maint() -> Result<()> {
//     // Get the number of available logical CPU cores
//     let num_cpus = rayon::current_num_threads();
//     println!("Number of CPU cores: {}", num_cpus);

//     // Execute the main logic in parallel using Rayon
//     (0..num_cpus).into_par_iter().for_each(|_| {
//         if let Err(err) = execute_main_logic() {
//             eprintln!("Error: {:?}", err);
//         }
//         println!("Thread {:?} finished", thread::current().id());
//     });

//     // let record = session.record().call_results();
//     // for result in record {
//     //     println!("{:?}\n", result);
//     // }
//     Ok(())
// }

// fn execute_main_logic() -> Result<()> {
//     let mut session = Session::<MinimalRuntime>::new()?;

//     // Load contract from file
//     let contract_path = Path::new("./flipper/target/ink/flipper.contract");
//     let contract = ContractBundle::load(contract_path).expect("Failed to load contract");

//     session.deploy_bundle(contract.clone(), "new", &["true"], NO_SALT, NO_ENDOWMENT)?;

//     let init_value: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
//     println!("Initial value: {}", init_value);

//     session.call("flip", NO_ARGS, NO_ENDOWMENT)??;

//     let value: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
//     println!("Value after flip: {}", value);

//     // let record = session.record().call_results();
//     // for result in record {
//     //     println!("{:?}\n", result);
//     // }

//     Ok(())
// }

