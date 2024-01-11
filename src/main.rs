use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use wasmi::*;
extern crate wabt;

use wabt::wasm2wat;

/// Simple cli to read files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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

fn get_wat(args: Args) -> Result<String, Box<dyn Error>> {
    let args_count = args.wat.is_some() as usize
        + args.contract.is_some() as usize
        + args.wasm.is_some() as usize;
    if args_count != 1 {
        return Err("Please specify exactly one of --wat, --contract, or --wasm".into());
    }
    let wat =
        match args.wat {
            Some(path) => get_wat_from_wat(path)?,
            None => {
                match args.contract {
                    Some(path) => get_wat_from_contract(path)?,
                    None => match args.wasm {
                        Some(path) => get_wat_from_wasm(path)?,
                        None => {
                            panic!("Please specify exactly one of --wat, --contract, or --wasm")
                        }
                    },
                }
            }
        };
    Ok(wat)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    // First step is to create the Wasm execution engine with some config.
    // In this example we are using the default configuration.
    let engine = Engine::default();
    let wat = get_wat(args)?;

    // Wasmi does not yet support parsing `.wat` so we have to convert
    // out `.wat` into `.wasm` before we compile and validate it.
    let wasm = wat::parse_str(wat)?;
    let module = Module::new(&engine, &mut &wasm[..])?;

    // All Wasm objects operate within the context of a `Store`.
    // Each `Store` has a type parameter to store host-specific data,
    // which in this case we are using `42` for.
    type HostState = u32;
    let mut store = Store::new(&engine, 42);
    let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
        println!("Got {param} from WebAssembly");
        println!("My host state is: {}", caller.data());
    });

    // In order to create Wasm module instances and link their imports
    // and exports we require a `Linker`.
    let mut linker = <Linker<HostState>>::new(&engine);
    // Instantiation of a Wasm module requires defining its imports and then
    // afterwards we can fetch exports by name, as well as asserting the
    // type signature of the function with `get_typed_func`.
    //
    // Also before using an instance created this way we need to start it.
    linker.define("host", "hello", host_hello)?;
    let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;
    let hello = instance.get_typed_func::<(), ()>(&store, "hello")?;

    // And finally we can call the wasm!
    hello.call(&mut store, ())?;
    Ok(())
}

#[cfg(test)]
#[path = "./tests/test.rs"]
mod tests;
