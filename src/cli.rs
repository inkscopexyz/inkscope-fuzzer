extern crate wabt;

use clap::Parser;
use wabt::wasm2wat;

use std::{
    error::Error,
    path::PathBuf,
};

/// Simple cli to read files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The wat file of the contract to fuzz test
    #[arg(long)]
    wat: Option<PathBuf>,

    /// The .contract file of the contract to fuzz test
    #[arg(long)]
    contract: Option<PathBuf>,

    /// The .wasm file of the contract to fuzz test
    #[arg(long)]
    wasm: Option<PathBuf>,
}

fn get_wat_from_wat(path: PathBuf) -> Result<String, Box<dyn Error>> {
    let wat = std::fs::read_to_string(path)?;
    Ok(wat)
}

fn get_wat_from_wasm(path: PathBuf) -> Result<String, Box<dyn Error>> {
    let wasm = std::fs::read(path)?;
    let wat = wasm2wat(wasm)?;
    Ok(wat)
}

fn get_wat_from_contract(path: PathBuf) -> Result<String, Box<dyn Error>> {
    let contract = std::fs::read_to_string(path)?;
    let contract: serde_json::Value = serde_json::from_str(&contract)?;
    let wat = contract["wasm"]["wat"].as_str().unwrap();
    Ok(wat.to_string())
}

pub fn get_wat(args: Args) -> Result<String, Box<dyn Error>> {
    let args_count = args.wat.is_some() as usize
        + args.contract.is_some() as usize
        + args.wasm.is_some() as usize;
    if args_count != 1 {
        return Err("Please specify exactly one of --wat, --contract, or --wasm".into());
    }
    let wat = match args.wat {
        Some(path) => get_wat_from_wat(path)?,
        None => {
            match args.contract {
                Some(path) => get_wat_from_contract(path)?,
                None => {
                    match args.wasm {
                        Some(path) => get_wat_from_wasm(path)?,
                        None => {
                            panic!("Please specify exactly one of --wat, --contract, or --wasm")
                        }
                    }
                }
            }
        }
    };
    Ok(wat)
}
