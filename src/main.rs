use wasmi::*;

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

pub struct TestHostState {
    data: u32,
}
// impl TestHostState {
//     pub fn new(memory: Vec<u8>, input: Vec<u8>, caller: AccountId) -> Self {
//         TestHostState {
//             memory,
//             input,
//             caller,
//         }
//     }

//     pub fn host_caller(mut context: Caller<'_, TestHostState>, pointer: u32, len: u32) {
//         let mut context = context.data_mut();
//         let slice = context.memory.as_mut_slice();
//         slice[pointer as usize..(pointer + len) as usize].copy_from_slice(&context.caller);
//     }
// }
/// This is the hashing algorithm used by the specific runtime

pub struct LoadedModule {
    pub module: Module,
    pub engine: Engine,
}

impl LoadedModule {
    /// Creates a new instance of `LoadedModule`.
    ///
    /// The inner Wasm module is checked not to have restricted WebAssembly proposals.
    /// Returns `Err` if the `code` cannot be deserialized or if it contains an invalid module.
    pub fn new(
        code: &[u8],
        determinism: bool,
        stack_limits: Option<StackLimits>,
    ) -> Result<Self, &'static str> {
        // NOTE: wasmi does not support unstable WebAssembly features. The module is implicitly
        // checked for not having those ones when creating `wasmi::Module` below.
        let mut config = Config::default();
        config
            .wasm_multi_value(false)
            .wasm_mutable_global(false)
            .wasm_sign_extension(true)
            .wasm_bulk_memory(false)
            .wasm_reference_types(false)
            .wasm_tail_call(false)
            .wasm_extended_const(false)
            .wasm_saturating_float_to_int(false)
            .floats(!determinism)
            .consume_fuel(false)
            .fuel_consumption_mode(FuelConsumptionMode::Eager);

        if let Some(stack_limits) = stack_limits {
            config.set_stack_limits(stack_limits);
        }

        let engine = Engine::new(&config);
        let module = Module::new(&engine, code).map_err(|_| "Can't load the module into wasmi!")?;

        // Return a `LoadedModule` instance with
        // __valid__ module.
        Ok(LoadedModule { module, engine })
    }
}

fn main() {
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
    let wasm = wat::parse_str(wat).unwrap();
    //let code = vec![0];
    let determinism = true;
    let contract = LoadedModule::new(&wasm, determinism, None).unwrap();

    type HostState = u32;

    let mut store = Store::new(&contract.engine, 45);
    let mut linker = Linker::new(&contract.engine);

    let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
        println!("Got {param} from WebAssembly");
        println!("My host state is: {}", caller.data());
    });
    linker.define("host", "hello", host_hello).unwrap();

    let memory = Memory::new(&mut store, MemoryType::new(40, Some(1024)).expect("")).expect("");

    linker
        .define("env", "memory", memory)
        .expect("We just created the Linker. It has no definitions with this name; qed");

    let instance = linker
        .instantiate(&mut store, &contract.module)
        .map_err(|_| "can't instantiate module with provided definitions")
        .unwrap();

    let started_instance = instance.start(&mut store);
    let hello = started_instance
        .unwrap()
        .get_typed_func::<(), ()>(&store, "hello")
        .unwrap();

    // And finally we can call the wasm!
    hello.call(&mut store, ()).unwrap();
}
