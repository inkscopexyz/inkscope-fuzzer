mod cli;
mod utils;
mod wasm;

use clap::Parser;

use cli::{
    get_wat,
    Args,
};

use wasm::{
    executor::*,
    host_state::HostState,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Get the wasm from the command line arguments
    let wat = get_wat(args)?;
    let wasm = wat::parse_str(wat)?;

    // Create a host state
    let host_state = HostState::builder()
        .input_buffer(vec![0xed, 0x4b, 0x9d, 0x1b])
        .build();

    // Create an executor
    let executor = Executor::new(wasm, host_state)?;
    let instance = executor.instance;
    let mut store = executor.store;

    let deploy = instance
        .get_typed_func::<(), ()>(&store, "deploy")
        .map_err(|e| {
            eprintln!("Error getting typed function 'deploy': {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    // And finally we can call the wasm!
    let result = deploy.call(&mut store, ());
    println!("Result: {:?}", result);
    let return_data = store.data().return_data.as_ref().unwrap();
    println!("Return data: {:?}", return_data);
    Ok(())
}

#[cfg(test)]
#[path = "./tests/test.rs"]
mod tests;
