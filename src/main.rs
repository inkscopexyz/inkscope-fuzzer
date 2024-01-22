mod cli;
mod module;
mod wasm;
mod utils;

use clap::Parser;
use module::LoadedModule;
use wasmi::{
    core::Trap,
    *,
};

use cli::{
    get_wat,
    Args,
};

use wasm::host_state::HostState;
use wasm::host_functions::HostFunctions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let wat = get_wat(args)?;

    // Wasmi does not yet support parsing `.wat` so we have to convert
    // out `.wat` into `.wasm` before we compile and validate it.
    let wasm = wat::parse_str(wat)?;
    let determinism = true;
    let contract = LoadedModule::new(&wasm, determinism, None).unwrap();

    let host_state = HostState::builder().input_buffer(vec![0xed, 0x4b, 0x9d, 0x1b]).build();

    let mut store = Store::new(&contract.engine, host_state);
    let mut linker = Linker::new(&contract.engine);
    let memory =
        Memory::new(&mut store, MemoryType::new(2, Some(16)).expect("")).expect("");
    store.data_mut().memory = Some(memory);

    let host_get_storage = Func::wrap(
        &mut store,
        |_caller: Caller<'_, HostState>,
         param: i32,
         param1: i32,
         param2: i32,
         param3: i32|
         -> i32 {
            println!("Hello from host_get_storage");
            println!("param: {}", param);
            println!("param1: {}", param1);
            println!("param2: {}", param2);
            println!("param3: {}", param3);
            1
        },
    );
    linker
        .define("seal1", "get_storage", host_get_storage)
        .unwrap();

    let hfn_seal2_set_storage = Func::wrap(
        &mut store,
        |mut ctx: Caller<'_, HostState>,
         key_ptr: u32,
         key_len: u32,
         value_ptr: u32,
         value_len: u32|
         -> Result<u32, Trap> {
            // TODO: this needs to be a true logging facility
            println!(
                "HOSTFN:: set_storage(key_ptr: 0x{:x}, key_len: 0x{:x}, value_ptr: 0x{:x}, value_len: 0x{:x})",
                key_ptr, key_len, value_ptr, value_len
            
            );
            let (memory, state) = ctx
                .data()
                .memory
                .expect("No memory")
                .data_and_store_mut(&mut ctx);

            state.seal2_set_storage(memory, key_ptr, key_len, value_ptr, value_len)
        },
    );
    
    linker
        .define("seal2", "set_storage", hfn_seal2_set_storage)
        .unwrap();

    let hfn_seal0_value_transferred = Func::wrap(
        &mut store,
        |mut ctx: Caller<'_, HostState>,
         out_ptr: u32,
         out_len_ptr: u32|
         -> Result<(), Trap> {
            println!(
                "HOSTFN:: value_transferred(out_ptr: 0x{:x}, out_len_ptr: 0x{:x})",
                out_ptr, out_len_ptr
            );

            let (memory, state) = ctx
                .data()
                .memory
                .expect("No memory")
                .data_and_store_mut(&mut ctx);

            state.seal0_value_transferred(memory, out_ptr, out_len_ptr)
        }
    );

    linker
        .define("seal0", "value_transferred", hfn_seal0_value_transferred)
        .unwrap();

    // TODO: make a macro! that generates the next 2 lines by something like this...
    // link_host_function!(state.seal0_input)
    let hfn_seal0_input = Func::wrap(
        &mut store,
        |mut ctx: Caller<'_, HostState>,
         buf_ptr: u32,
         buf_len_ptr: u32|
         -> Result<(), Trap> {
            // TODO: this needs to be a true logging facility
            println!(
                "HOSTFN:: input(buf_ptr: 0x{:x}, buf_len_ptr: 0x{:x})",
                buf_ptr, buf_len_ptr
            );
            let (memory, state) = ctx
                .data()
                .memory
                .expect("No memory")
                .data_and_store_mut(&mut ctx);

            state.seal0_input(memory, buf_ptr, buf_len_ptr)
        },
    );
    linker.define("seal0", "input", hfn_seal0_input).unwrap();

    let hfn_seal0_seal_return = Func::wrap(
        &mut store,
        |mut ctx: Caller<'_, HostState>,
         flags: u32,
         out_ptr: u32,
         out_len: u32|
         -> Result<(), Trap> {

            println!(
                "HOSTFN:: seal_return(flags: 0x{:x}, out_ptr: 0x{:x}, out_len: 0x{:x})",
                flags, out_ptr, out_len
            );

            let (memory, state) = ctx
                .data()
                .memory
                .expect("No memory")
                .data_and_store_mut(&mut ctx);

            state.seal0_seal_return(memory, flags, out_ptr, out_len)

        },);

        linker.define("seal0", "seal_return", hfn_seal0_seal_return).unwrap();

    linker
        .define("env", "memory", memory)
        .expect("We just created the Linker. It has no definitions with this name; qed");

    let instance = linker
        .instantiate(&mut store, &contract.module)
        .map_err(|e| {
            eprintln!("Error: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })
        .and_then(|instance| {
            println!("Instance created!");
            instance.start(&mut store).map_err(|e| {
                eprintln!("Error starting instance: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })
        })?;

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
