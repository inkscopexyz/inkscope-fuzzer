mod ext_env;
mod flipper;
use ext_env::*;
use flipper::FLIPPER_WAT;
use wasmi::*;
use wasmi::core::Trap;

/* Different parachains may implement pallet-contract in different ways. There are a number of types and parameters that could vary,
rather slightly, bwtween parachains. We should have a way to configure the fuzzer to generate different emulated environments for
different parachains.

Whe should have our "T-like" Config that defines all these parameters and types. The Fuzzer will then depend? on that like so: Fuzzer<C: Config>
We will have different configs resembling different real parachains environments.

Config{
    type AccountId = [u8; 32];
    type Hash = [u8; 32];
    type Balance = u128;
    type CodeType = Vec<u8>;
    type AllowDeprecatedInterface = bool;
    type AllowUnstableInterface = bool;
    type Determinism = bool;  // If true the execution should be deterministic and hence no indeterministic instructions are allowed.
    type Schedule = Schedule;
    type MaxCodeSize = MaxCodeSize;
    type MaxMemoryPages = MaxMemoryPages;
    type MaxTotalLength = MaxTotalLength;
    type MaxSubjectLen = MaxSubjectLen;
    type MaxCodeSize = MaxCodeSize;
    type MaxGas = MaxGas;
    type MaxValueSize = MaxValueSize;
    type MaxStackHeight = MaxStackHeight;
    type MaxDepth = MaxDepth;
    type MaxTopics = MaxTopics;
    type MaxEventSize = MaxEventSize;
    type MaxReads = MaxReads;
    type MaxWrites = MaxWrites;
    type WeightPrice = WeightPrice;
    type WeightPrice = WeightPrice;
}

Example types that may vary between parachains: AccountId, Hash, Balance, MaxSizeOfCode, ...
Hash algorithm (for example to calculate the codehash) may vary.

WorldState
Then I magine we should have a world state containing a snapshot of all the emulated world state. Worldstate can have functions to modify it. Give balance,
set accounts, etc.
Should it be an overlay over actual blockchain?
Accessing an account that does not exist?
Should we download it from a block?
Should we return an error?

An account should have a balance, storage, codehash, etc.
type Storage = HashMap<vec<u8>, vec<u8>>;
Account{
    balance: Balance,
    storage: Storage,
    ..
}

The world state contains a snapshot of all the accounts, code, etc. It should have functions to modify it. Give balance, etc.
WorldState {
    accounts: HashMap<AccountId, Account>,
    code: HashMap<Hash, Code>,
    ...
}


The emulated execution starts with a call() or a deploy(). The wasm engine stuff must be pregenerated and stored in the world state.
A seed will suffice prandomly generate all the inputs needed for a run.

CallFrame{
    module: Module,
    instance: Instance,
    memory: Memory,
    input: vec<u8>,
}

Trace{
    world_state: WorldState,
    call_stack: Vec<CallFrame>,
}


There should be a host functions trait somewhere that implements all the methods that could be called from the wasm code.
The result from input() will depend on the seed and on some static information gathered from the current contract (constants or the abi)

...

Now we need to rewrite that doc in a more INK/Substrate way.

*/






/// Stores the input passed by the caller into the supplied buffer.
///
/// The value is stored to linear memory at the address pointed to by `out_ptr`.
/// `out_len_ptr` must point to a u32 value that describes the available space at
/// `out_ptr`. This call overwrites it with the size of the value. If the available
/// space at `out_ptr` is less than the size of the value a trap is triggered.
///


fn host_input_fn(mut ctx: Caller<'_, HostState>, buf_ptr: i32, buf_len_ptr: u32) -> Result<(), Trap> {
    //TODO: this needs to be a true logging facility
    println!("HOSTFN:: input(buf_ptr: 0x{:x}, buf_len_ptr: 0x{:x})", buf_ptr, buf_len_ptr);
=
    // // TODO move this to a function if possible-> let memory = get_mem(ctx);
    let state = ctx.data_mut();
    let memory = state.memory.ok_or(Trap::new("No memory"))?;
    let memory = memory.data_mut(ctx.as_context_mut());


    // let buf_len = state.decode_from_memory_as::<u32>(memory, buf_len_ptr as u32);
    // //println!("HOSTFN:: read buf_len: {}", buf_len);


Ok(())
    //0xed4b9d1b
    // let wr = memory.write(&mut context, buf_ptr as usize, &[0xed, 0x4b, 0x9d, 0x1b]);
    // //[0xed, 0x4b, 0x9d, 0x1b]
    // println!("write1 result: {:?}", wr);
    // let wr = memory.write(&mut context, buf_len_ptr as usize, &[0x04, 0, 0, 0]);
    // println!("write2 result: {:?}", wr);
    // // &caller.data_mut().memory.unwrap().write(
    // //     caller,
    // //     param as usize,
    // //     &[0xed, 0x4b, 0x9d, 0x1b],
    // // );
    // let mut buffer: [u8; 32] = [0; 32];
    // let rd =
    //     context
    //         .data()
    //         .memory
    //         .unwrap()
    //         .read(&mut context, buf_ptr as usize, &mut buffer);
    // println!("read result: {:?}", rd);
    // println!("memory {:?}", buffer);

}


fn main() {
    // Wasmi does not yet support parsing `.wat` so we have to convert
    // out `.wat` into `.wasm` before we compile and validate it.
    let wasm = wat::parse_str(FLIPPER_WAT).unwrap();
    //let code = vec![0];
    let determinism = true;
    let contract = LoadedModule::new(&wasm, determinism, None).unwrap();

    let mut host_state =
        HostState {
            input_buffer: vec![],
            caller: [0; 32],
            value_transferred: 0,
            memory: None,
        };
    let mut store = Store::new(&contract.engine, host_state);
    let mut linker = Linker::new(&contract.engine);
    let memory = Memory::new(&mut store, MemoryType::new(2, Some(16)).expect("")).expect("");
    store.data_mut().memory = Some(memory);

    let host_get_storage = Func::wrap(
        &mut store,
        |caller: Caller<'_, HostState>, param: i32, param1: i32, param2: i32, param3: i32| -> i32 {
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

    let host_set_storage = Func::wrap(
        &mut store,
        |caller: Caller<'_, HostState>, param: i32, param1: i32, param2: i32, param3: i32| -> i32 {
            println!("Hello from host_set_storage");
            println!("param: {}", param);
            println!("param1: {}", param1);
            println!("param2: {}", param2);
            println!("param3: {}", param3);
            2
        },
    );
    linker
        .define("seal2", "set_storage", host_set_storage)
        .unwrap();

    let host_value_transferred = Func::wrap(
        &mut store,
        |caller: Caller<'_, HostState>, param: i32, param1: i32| {
            println!("Hello from transferred");
            println!("param: {}", param);
            println!("param1: {}", param1);
        },
    );
    linker
        .define("seal0", "value_transferred", host_value_transferred)
        .unwrap();

    let host_input = Func::wrap(
        &mut store, host_input_fn);

    linker.define("seal0", "input", host_input).unwrap();

    let host_seal_return = Func::wrap(
        &mut store,
        |caller: Caller<'_, HostState>, flags: i32, data_ptr: i32, data_len: i32| {
            println!("Hello from seal_return");
            println!("flags: {}", flags);
            println!("data_ptr: {}", data_ptr);
            println!("data_len: {}", data_len);
        },
    );
    linker
        .define("seal0", "seal_return", host_seal_return)
        .unwrap();

    linker
        .define("env", "memory", memory)
        .expect("We just created the Linker. It has no definitions with this name; qed");

    let res_instance: Result<InstancePre, Error> = linker.instantiate(&mut store, &contract.module);
    match &res_instance {
        Ok(instance) => {
            println!("Instance created!");
        }
        Err(e) => println!("Error: {}", e),
    };
    let instance = res_instance.unwrap();
    let started_instance = instance.start(&mut store);
    let deploy = started_instance
        .unwrap()
        .get_typed_func::<(), ()>(&store, "deploy")
        .unwrap();

    // And finally we can call the wasm!
    deploy.call(&mut store, ()).unwrap();
}
