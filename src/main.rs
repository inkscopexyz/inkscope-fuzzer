use std::error::Error;
use wasmi::*;
use wasmi::{Config as WasmiConfig, Linker, Memory, MemoryType, StackLimits, Store};

fn main() -> Result<(), Box<dyn Error>> {
    // First step is to create the Wasm execution engine with some config.
    // In this example we are using the default configuration.
    let wat = r#"
        (module
            (import "host" "hello" (func $host_hello (param i32)))
            (func (export "hello")
                (call $host_hello (i32.const 3))
            )
        )
    "#;
    // Wasmi does not yet support parsing `.wat` so we have to convert
    // out `.wat` into `.wasm` before we compile and validate it.
    let wasm = wat::parse_str(wat)?;

    let determinism = false; // We assume that the code is not using floating point determinism
    let stack_limits = StackLimits::default();
    let memory_limits = (0, 200); //TODO: set this to a reasonable value
    type HostState = u32;
    let host_state: HostState = 42;
    let consume_fuel = false; //TODO:This should be set to true to match pallet_contracts

    let mut config = WasmiConfig::default();
    config
        .wasm_multi_value(false)
        .wasm_mutable_global(false)
        .wasm_sign_extension(true)
        .wasm_bulk_memory(false)
        .wasm_reference_types(false)
        .wasm_tail_call(false)
        .wasm_extended_const(false)
        .wasm_saturating_float_to_int(false)
        .floats(determinism)
        .consume_fuel(consume_fuel)
        .fuel_consumption_mode(FuelConsumptionMode::Eager)
        .set_stack_limits(stack_limits);

    let engine = Engine::new(&config);
    let module =
        Module::new(&engine, &wasm[..]).map_err(|_| "Can't load the module into wasmi!")?;

    let mut store = Store::new(&engine, host_state);
    let mut linker: Linker<HostState> = Linker::new(&engine);

    // Here we allocate this memory in the _store_. It allocates _inital_ value, but allows it
    // to grow up to maximum number of memory pages, if necessary.

    let memory = Memory::new(
        &mut store,
        MemoryType::new(memory_limits.0, Some(memory_limits.1)).expect(""),
    )
    .expect("");

    linker
        .define("env", "memory", memory)
        .expect("We just created the Linker. It has no definitions with this name; qed");

    let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
        println!("Got {param} from WebAssembly");
        println!("My host state is: {}", caller.data());
    });

    linker.define("host", "hello", host_hello)?;

    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|_| "can't instantiate module with provided definitions")?;
    // let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;
    let started_instance = instance.start(&mut store)?;
    let hello = started_instance.get_typed_func::<(), ()>(&store, "hello")?;

    // And finally we can call the wasm!
    hello.call(&mut store, ())?;
    Ok(())
}
