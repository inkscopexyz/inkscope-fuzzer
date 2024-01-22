use super::{
    host_state::HostState,
    module::LoadedModule,
    host_functions::HostFunctions,
};
use wasmi::{
    core::Trap, Caller, Func, Instance, Linker, Memory, MemoryType, Store
};

pub struct Executor {
    pub instance: Instance,
    pub store: Store<HostState>,
}

impl Executor {
    pub fn new(wasm: Vec<u8>, host_state: HostState) -> Result<Self, Box<dyn std::error::Error>> {
        // Load wasm into wasmi
        let contract = LoadedModule::new(&wasm, true, None).unwrap();
        
        // Create a store and linker
        let mut store = Store::new(&contract.engine, host_state);
        let mut linker = Linker::new(&contract.engine);

        // Create the linear memory
        // TODO: Check scan_imports fn in polkadotsdk to get correct memory size
        let memory =
            Memory::new(&mut store, MemoryType::new(2, Some(16)).expect("")).expect("");
        store.data_mut().memory = Some(memory);

        // Define and link host functions
        // TODO: move this code to somewhere else or create a macro
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
        Ok(Self { instance, store})
    }
}
