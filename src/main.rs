mod ext_env;
mod flipper;
use ext_env::*;
use flipper::FLIPPER_WAT;
use wasmi::*;
use wasmi::core::Trap;
use std::convert::TryFrom;
use std::collections::HashMap;

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


fn host_input_fn(mut ctx: Caller<'_, HostState>, buf_ptr: u32, buf_len_ptr: u32) -> Result<(), Trap> {
    //TODO: this needs to be a true logging facility
    println!("HOSTFN:: input(buf_ptr: 0x{:x}, buf_len_ptr: 0x{:x})", buf_ptr, buf_len_ptr);
    let (memory, state ) = ctx.data().memory.expect("No memory").data_and_store_mut(&mut ctx);

    let buf_len = state.decode_from_memory::<u32>(memory, buf_len_ptr).unwrap();
    println!("HOSTFN:: read buf_len: {}", buf_len);

    // TODO generate approiate inpud using host state and seed and abi and whatever
    let input = state.get_input();
    let input_len = u32::try_from(input.len()).expect("Buffer length must be less than 4Gigs");
    
    state.write_to_memory(memory, buf_ptr, input)?;
    state.encode_to_memory(memory, buf_len_ptr, input_len)
}


/// Set the value at the given key in the contract storage.
///
/// The key and value lengths must not exceed the maximums defined by the contracts module
/// parameters. Specifying a `value_len` of zero will store an empty value.
///
/// # Parameters
///
/// - `key_ptr`: pointer into the linear memory where the location to store the value is placed.
/// - `key_len`: the length of the key in bytes.
/// - `value_ptr`: pointer into the linear memory where the value to set is placed.
/// - `value_len`: the length of the value in bytes.
///
/// # Return Value
///
/// Returns the size of the pre-existing value at the specified key if any. Otherwise
/// `SENTINEL` is returned as a sentinel value.
fn host_set_storage(
    mut ctx: Caller<'_, HostState>, 
    key_ptr: u32,
    key_len: u32,
    value_ptr: u32,
    value_len: u32,
)  -> Result<u32, Trap> {
    //TODO: this needs to be a true logging facility
    println!("HOSTFN:: set_storage(key_ptr: 0x{:x}, key_len: 0x{:x}, value_ptr: 0x{:x}, value_len: 0x{:x})", key_ptr, key_len, value_ptr, value_len);
    let (memory, state ) = ctx.data().memory.expect("No memory").data_and_store_mut(&mut ctx);
    
    state.set_storage(memory, key_ptr, key_len, value_ptr, value_len)
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
            storage: HashMap::new(),
            input_buffer: vec![  0xed, 0x4b, 0x9d, 0x1b ],
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
        host_set_storage,
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
