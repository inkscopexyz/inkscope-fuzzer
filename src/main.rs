use anyhow::{anyhow, Result};

use drink::{
    frame_support::{
        pallet_prelude::{Encode, Decode}, print, sp_runtime::traits::{
            BlakeTwo256, Bounded, SaturatedConversion, UniqueSaturatedInto, Hash as HashTrait, TrailingZeroInput
        }, weights::constants::WEIGHT_PROOF_SIZE_PER_KB, Hashable
    }, frame_system::{self, offchain::{Account, SendSignedTransaction}}, pallet_contracts::{
        AddressGenerator, ContractExecResult, DefaultAddressGenerator, Determinism,
    }, runtime::{
        pallet_contracts_debugging::TracingExt, AccountIdFor, HashFor, MinimalRuntime,
    }, sandbox::{self, Snapshot}, session::{self, Session, NO_ARGS, NO_ENDOWMENT, NO_SALT}, BalanceOf, ContractBundle, DispatchError, SandboxConfig, Weight
};
use fastrand::Rng;
use hex;
use log::{debug, error, info, trace};
use env_logger;
use parity_scale_codec::Compact as ScaleCompact;
use rayon::{prelude::*, result};
use scale_info::{
    form::PortableForm, IntoPortable, PortableType, TypeDef, TypeDefArray,
    TypeDefBitSequence, TypeDefCompact, TypeDefComposite, TypeDefPrimitive,
    TypeDefSequence, TypeDefTuple, TypeDefVariant,
};
use std::{
    any::Any, cell::RefCell, collections::{HashMap, HashSet}, default, hash::{DefaultHasher, Hash as StdHash, Hasher}, path::{Path, PathBuf}, ptr::hash, thread
};


//TODO: add this to drink/runtime.rs
pub type HashingFor<R> = <R as frame_system::Config>::Hashing;

//This defines all the configurable types based on the current runtime: MinimalRuntime
type Balance = BalanceOf<MinimalRuntime>;
type AccountId = AccountIdFor<MinimalRuntime>;
type Hash = HashFor<MinimalRuntime>;
type CodeHash = HashFor<MinimalRuntime>;
type Hashing = HashingFor<MinimalRuntime>;
type TraceHash = u64;

struct RuntimeFuzzer {
    rng: RefCell<Rng>,
    contract_path: PathBuf,
    contract: ContractBundle,
    cache: HashMap<TraceHash, Snapshot>,
    //Settings
    budget: Balance,
    accounts: Vec<AccountId>,
    ignore_pure_messages: bool,
    max_sequence_type_size: u8,
    max_number_of_transactions: usize,
}

impl RuntimeFuzzer {
    fn new(contract_path: PathBuf) -> Self {
        //TODO! Do it right
        // let default_callers: Vec<AccountId> = vec![
        //     "Alice".into(),
        //     "Bob".into(),
        //     "Charlie".into(),
        //     "Dave".into(),
        //     "Eve".into(),
        //     "Ferdinand".into(),
        //     "Gina".into(),
        //     "Hank".into(),
        //     "Ivan".into(),
        //     "Jenny".into(),
        // ];

        let default_callers: Vec<AccountId> = vec![AccountId::new([41u8; 32])];

        Self {
            rng: RefCell::new(Rng::new()),
            contract_path: contract_path.clone(),
            contract: ContractBundle::load(&contract_path)
                .expect("Failed to load contract"),
            cache: HashMap::new(),
            budget: Balance::max_value() / 1000,
            accounts: default_callers,
            ignore_pure_messages: true,
            max_sequence_type_size: 100,
            max_number_of_transactions: 50,
        }
    }

    fn generate_argument(&self, type_def: &TypeDef<PortableForm>) -> Result<Vec<u8>> {
        match type_def {
            TypeDef::Composite(composite) => self.generate_composite(composite),
            TypeDef::Array(array) => self.generate_array(array),
            TypeDef::Tuple(tuple) => self.generate_tuple(tuple),
            TypeDef::Sequence(sequence) => self.generate_sequence(sequence),
            TypeDef::Variant(variant) => self.generate_variant(variant),
            TypeDef::Primitive(primitive) => self.generate_primitive(primitive),
            TypeDef::Compact(compact) => self.generate_compact(compact),
            TypeDef::BitSequence(bit_sequence) => {
                self.generate_bit_sequence(bit_sequence)
            }
        }
    }

    #[inline(always)]
    fn get_typedef(&self, type_id: u32) -> Result<&TypeDef<PortableForm>> {
        match self
            .contract
            .transcoder
            .metadata()
            .registry()
            .resolve(type_id)
        {
            Some(type_def) => Ok(&type_def.type_def),
            None => Err(anyhow::anyhow!("Type not found")),
        }
    }

    fn generate_composite(
        &self,
        composite: &TypeDefComposite<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        for field in &composite.fields {
            let field_type_def = self.get_typedef(field.ty.id)?;
            let mut field_encoded = self.generate_argument(&field_type_def)?;
            encoded.append(&mut field_encoded);
        }
        Ok(encoded)
    }

    fn generate_array(&self, array: &TypeDefArray<PortableForm>) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        //No length is included in the encoding as it is known at decoding
        let param_type_def = self.get_typedef(array.type_param.id)?;
        for i in 0..array.len {
            let mut param_encoded = self.generate_argument(param_type_def)?;
            encoded.append(&mut param_encoded);
        }
        Ok(encoded)
    }

    fn generate_tuple(&self, tuple: &TypeDefTuple<PortableForm>) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        //Encode the length in compact form
        ScaleCompact(tuple.fields.len() as u32).encode_to(&mut encoded);

        for field in &tuple.fields {
            let field_type = self.get_typedef(field.id)?;
            let mut field_encoded = self.generate_argument(field_type)?;
            encoded.append(&mut field_encoded);
        }
        Ok(encoded)
    }

    fn generate_sequence(
        &self,
        sequence: &TypeDefSequence<PortableForm>,
    ) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        // Fuzz a sequece size and encode it in compact form
        let size = self.rng.borrow_mut().u8(0..self.max_sequence_type_size);
        ScaleCompact(size).encode_to(&mut encoded);

        let param_type_def = self.get_typedef(sequence.type_param.id)?;
        for i in 0..size {
            let mut param_encoded = self.generate_argument(param_type_def)?;
            encoded.append(&mut param_encoded);
        }
        Ok(encoded)
    }

    fn generate_variant(
        &self,
        variant: &TypeDefVariant<PortableForm>,
    ) -> Result<Vec<u8>> {
        //TODO FIXME REview this code
        let selected_variant = self
            .rng
            .borrow_mut()
            .choice(&variant.variants)
            .expect("No variants");
        let mut encoded = selected_variant.index.encode();
        for field in &selected_variant.fields {
            let field_type = self.get_typedef(field.ty.id)?;
            let mut field_encoded = self.generate_argument(&field_type)?;
            encoded.append(&mut field_encoded);
        }
        Ok(encoded)
    }

    fn generate_primitive(&self, primitive: &TypeDefPrimitive) -> Result<Vec<u8>> {
        match primitive {
            TypeDefPrimitive::Bool => self.generate_bool(),
            TypeDefPrimitive::Char => {
                Err(anyhow::anyhow!("scale codec not implemented for char"))
            }
            TypeDefPrimitive::Str => self.generate_str(),
            TypeDefPrimitive::U8 => self.generate_u8(),
            TypeDefPrimitive::U16 => self.generate_u16(),
            TypeDefPrimitive::U32 => self.generate_u32(),
            TypeDefPrimitive::U64 => self.generate_u64(),
            TypeDefPrimitive::U128 => self.generate_u128(),
            TypeDefPrimitive::U256 => self.generate_u256(),
            TypeDefPrimitive::I8 => self.generate_i8(),
            TypeDefPrimitive::I16 => self.generate_i16(),
            TypeDefPrimitive::I32 => self.generate_i32(),
            TypeDefPrimitive::I64 => self.generate_i64(),
            TypeDefPrimitive::I128 => self.generate_i128(),
            TypeDefPrimitive::I256 => self.generate_i256(),
        }
    }

    fn generate_compact(
        &self,
        compact: &TypeDefCompact<PortableForm>,
    ) -> Result<Vec<u8>> {
        let param_typedef = self.get_typedef(compact.type_param.id)?;
        match param_typedef {
            TypeDef::Primitive(primitive) => self.generate_compact_primitive(primitive),
            TypeDef::Composite(composite) => self.generate_compact_composite(composite),
            _ => Err(anyhow::anyhow!(
                "Compact type must be a primitive or a composite type"
            )),
        }
    }
    fn generate_compact_primitive(
        &self,
        primitive: &TypeDefPrimitive,
    ) -> Result<Vec<u8>> {
        match primitive {
            TypeDefPrimitive::U8 => self.generate_compact_u8(),
            TypeDefPrimitive::U16 => self.generate_compact_u16(),
            TypeDefPrimitive::U32 => self.generate_compact_u32(),
            TypeDefPrimitive::U64 => self.generate_compact_u64(),
            TypeDefPrimitive::U128 => self.generate_compact_u128(),
            _ => Err(anyhow::anyhow!(
                "Compact encoding not supported for {:?}",
                primitive
            )),
        }
    }

    fn generate_compact_u8(&self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.rng.borrow_mut().u8(..)).encode())
    }

    fn generate_compact_u16(&self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.rng.borrow_mut().u16(..)).encode())
    }

    fn generate_compact_u32(&self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.rng.borrow_mut().u32(..)).encode())
    }

    fn generate_compact_u64(&self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.rng.borrow_mut().u64(..)).encode())
    }

    fn generate_compact_u128(&self) -> Result<Vec<u8>> {
        Ok(ScaleCompact(self.rng.borrow_mut().u128(..)).encode())
    }

    fn generate_compact_composite(
        &self,
        _composite: &TypeDefComposite<PortableForm>,
    ) -> Result<Vec<u8>> {
        todo!("Compact encoding for composite types not supported IMPLEEEMEEENT MEEEEEEEEE!")
    }

    fn generate_bit_sequence(
        &self,
        bit_sequence: &TypeDefBitSequence<PortableForm>,
    ) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("Bitsequence currently not supported"))
    }

    fn generate_bool(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().bool().encode())
    }

    fn generate_str(&self) -> Result<Vec<u8>> {
        //TODO: choose for  set of predeined strings extracted from the contract and other sources
        Ok(["A"].repeat(self.rng.borrow_mut().usize(1..100)).encode())
    }

    fn generate_u8(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().u8(..).encode())
    }

    fn generate_u16(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().u16(..).encode())
    }

    fn generate_u32(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().u32(..).encode())
    }

    fn generate_u64(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().u64(..).encode())
    }

    fn generate_u128(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().u128(..).encode())
    }

    fn generate_u256(&self) -> Result<Vec<u8>> {
        //TODO: We can encode a random u256 value
        Err(anyhow::anyhow!("U256 currently not supported"))
    }

    fn generate_i8(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().i8(..).encode())
    }

    fn generate_i16(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().i16(..).encode())
    }

    fn generate_i32(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().i32(..).encode())
    }

    fn generate_i64(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().i64(..).encode())
    }

    fn generate_i128(&self) -> Result<Vec<u8>> {
        Ok(self.rng.borrow_mut().i128(..).encode())
    }

    fn generate_i256(&self) -> Result<Vec<u8>> {
        //TODO: We can encode a random i256 value
        Err(anyhow::anyhow!("I256 currently not supported"))
    }

    fn generate_arguments(&self, args: Vec<&TypeDef<PortableForm>>) -> Result<Vec<u8>> {
        let mut encoded_args = Vec::new();
        for arg in args {
            let mut arg_encoded = self.generate_argument(&arg)?;
            encoded_args.append(&mut arg_encoded);
        }
        Ok(encoded_args)
    }

    // Generates a fuzzed constructor to be prepended in the trace
    fn generate_constructor(&self) -> FuzzerDeploy {
        let transcoder = &self.contract.transcoder;
        let metadata = transcoder.metadata();
        let constructors = metadata.spec().constructors();

        let selected_constructor = self
            .rng
            .borrow_mut()
            .choice(constructors)
            .expect("No constructors");
        let selectec_args_spec = selected_constructor.args();

        let selector = selected_constructor.selector();
        let mut encoded = selector.to_bytes().to_vec();

        let selected_args_type_defs = selectec_args_spec
            .iter()
            .map(|arg| self.get_typedef(arg.ty().ty().id).unwrap())
            .collect();

        let mut encoded_args = self.generate_arguments(selected_args_type_defs).unwrap();
        encoded.append(&mut encoded_args);
        let caller = self.generate_caller();
        let endowment = self.generate_endowment(&caller);
        FuzzerDeploy {
            caller,
            endowment,
            contract_bytes: self.contract.wasm.clone(),
            data: encoded,
            salt: Default::default(),
        }
    }

    // Generates a fuzzed message to be added in the trace
    fn generate_message(&self, callee: AccountId) -> FuzzerMessage {
        let transcoder = &self.contract.transcoder;
        let metadata = transcoder.metadata();

        // Keep only the messages that mutate the state unless ignore_pure_messages is false
        let mut messages = metadata
            .spec()
            .messages()
            .iter()
            .filter(|m| m.mutates() || !self.ignore_pure_messages);

        let count = messages.clone().count();
        let selected_message_idx = self.rng.borrow_mut().usize(0..count);
        let selected_message = messages.nth(selected_message_idx).unwrap();
        let selectec_args_spec = selected_message.args();

        let selector = selected_message.selector();
        let mut input = selector.to_bytes().to_vec();

        let selected_args_type_defs = selectec_args_spec
            .iter()
            .map(|arg| self.get_typedef(arg.ty().ty().id).unwrap())
            .collect();

        let mut encoded_args = self.generate_arguments(selected_args_type_defs).unwrap();
        input.append(&mut encoded_args);
        let caller = self.generate_caller();
        let endowment = self.generate_endowment(&caller);
        FuzzerMessage {
            caller,
            callee,
            endowment,
            input,
        }
    }

    //This should generate a random account id from the set of potential callers
    fn generate_caller(&self) -> AccountId {
        self.rng
            .borrow_mut()
            .choice(&self.accounts)
            .expect("You need to configure some potential callers")
            .clone()
    }
    fn generate_endowment(&self, _caller: &AccountId) -> Balance {
        // TODO! This should be a sensible value related to the balance of the caller
        let max_endowment: u128 = self.budget.saturated_into::<u128>();
            self.rng.borrow_mut().u128(0..max_endowment) as Balance
    }

    fn initialize_state(
        &self,
        session: &mut Session<MinimalRuntime>,
        _trace: &[FuzzerCall],
    ) -> Result<()> {
        debug!("Setting initial state. Give initial budget to caller addresses.");
        // Assigning initial budget to caller addresses
        let sandbox = session.sandbox();
        for account in &self.accounts {
            debug!("  Mint {} to {}", self.budget, account);
            sandbox
                .mint_into(account.clone(), self.budget)
                .map_err(|e| anyhow::anyhow!("Error minting into account: {:?}", e))?;
        }
        Ok(())
    }

    fn execute_call(
        &self,
        session: &mut Session<MinimalRuntime>,
        calls: &[FuzzerCall],
    ) -> Result<()> {
        // TODO! We need to control this config value from s different place
        let gas_limit = Weight::max_value() / 4;

        let call = match calls.last() {
            Some(call) => call,
            None => anyhow::bail!("No calls to execute"),
        };

        let result = match call {
            FuzzerCall::Message(message) => {
                println!("Sending message with data {:?}", message);

                session
                    .sandbox()
                    .call_contract(
                        message.caller.clone(),
                        message.endowment,
                        message.input.clone(),
                        message.caller.clone(),
                        gas_limit,
                        None,
                        Determinism::Enforced,
                    )
                    .result
                    .map_err(|e| anyhow::anyhow!("Error executing message: {:?}", e))?
            }
            FuzzerCall::Deploy(deploy) => {
                info!("Deploying contract with data {:?}", deploy);

                let deployment_result = session
                    .sandbox()
                    .deploy_contract(
                        deploy.contract_bytes.clone(),
                        0,
                        deploy.data.clone(),
                        deploy.salt.clone(),
                        deploy.caller.clone(),
                        gas_limit,
                        None,
                    );
                    println!("Deployment result: {:?}", deployment_result.result);
                    deployment_result.result
                    .map_err(|e| {println!("ERR {:?}", e); anyhow::anyhow!("Error executing deploy: {:?}", e)})?
                    .result
            }
        };

        println!("Result: {:?}", result);
        // results.flags ? revert?

        self.check_properties(&session)
    }

    fn check_properties(&self, session: &Session<MinimalRuntime>) -> Result<()> {
        //TODO! We need to check the properties of the contract
        Ok(())
    }


    fn run(&mut self) -> Result<()> {
        let mut session = Session::<MinimalRuntime>::new()?;
        let mut trace = Vec::new();
        let mut current_state = None;

        // Initialize the state:
        //    - Assigning initial budget to caller addresses
        //STEP
        match self.cache.get(&hash_trace(&mut trace)) {
            Some(snapshot) => {
                println! ( "Cache hit");
                // The trace is already in the cache set current state
                current_state = Some(snapshot);
            }
            None => {
                // The trace was not in the cache, apply the previous state if any
                if let Some(snapshot) = current_state {
                    session.sandbox().restore_snapshot(snapshot.clone());
                }
                // Execute the given action
                self.initialize_state(&mut session, &trace)?;

                // If the closure returned Ok(()) then store the new state in the cache
                self.cache
                    .insert(hash_trace(&trace), session.sandbox().take_snapshot());
                current_state = None;
            }
        };

        // Deploy the main contract to be fuzzed using a random constructor with fuzzed argumets
        let constructor = self.generate_constructor();
        let contract_address = constructor.calculate_address();
        println!("Contract address: {:?}", contract_address);
        let call_deploy = FuzzerCall::Deploy(constructor);
        trace.push(call_deploy);

        //STEP
        match self.cache.get(&hash_trace(&mut trace)) {
            Some(snapshot) => {
                println!( "Cache hit");

                // The trace is already in the cache set current state
                current_state = Some(snapshot);
            }
            None => {
                println!( "Cache miss");
                // The trace was not in the cache, apply the previous state if any
                if let Some(snapshot) = current_state {
                    println!( "cloning saved state from previous cache");

                    session.sandbox().restore_snapshot(snapshot.clone());
                }
                // Execute the given action
                self.execute_call(&mut session, &trace)?;

                // If the closure returned Ok(()) then store the new state in the cache
                self.cache
                    .insert(hash_trace(&trace), session.sandbox().take_snapshot());
                current_state = None;
            }
        };

        // Randomly choose how many fuzzed messages to send
        let iterations = self
            .rng
            .borrow_mut()
            .usize(..self.max_number_of_transactions);

        for _i in 0..iterations {
            println!("iteration: {}", _i);
            trace.push(FuzzerCall::Message(self.generate_message(contract_address.clone())));

            //STEP
            match self.cache.get(&hash_trace(&mut trace)) {
                Some(snapshot) => {
                    println! ( "Cache hit");

                    // The trace is already in the cache set current state
                    current_state = Some(snapshot);
                }
                None => {
                    // The trace was not in the cache, apply the previous state if any
                    if let Some(snapshot) = current_state {
                        session.sandbox().restore_snapshot(snapshot.clone());
                    }
                    // Execute the given action
                    self.execute_call(&mut session, &trace)?;

                    // If the closure returned Ok(()) then store the new state in the cache
                    self.cache
                        .insert(hash_trace(&trace), session.sandbox().take_snapshot());
                    current_state = None;
                }
            };
        }
        Ok(())
    }



}

#[derive(StdHash)]
enum FuzzerCall {
    Deploy(FuzzerDeploy),
    Message(FuzzerMessage),
}

#[derive(StdHash, Debug)]
struct FuzzerDeploy {
    caller: AccountId,
    endowment: Balance,
    contract_bytes: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>,
}
impl FuzzerDeploy {
    fn calculate_code_hash(&self) -> CodeHash {
        Hashing::hash(&self.contract_bytes)
    }

    fn calculate_address(&self) -> AccountId {
        let deploying_address = &self.caller;
        let code_hash: CodeHash = self.calculate_code_hash();
        let input_data = &self.data;
        let salt = &self.salt;
        
        let entropy = (b"contract_addr_v1", deploying_address, code_hash, input_data, salt)
        .using_encoded(Hashing::hash);
        Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
        .expect("infinite length input; no invalid inputs for type; qed")

        // DefaultAddressGenerator::contract_address(
        //     &deploying_address,
        //     &code_hash,
        //     &input_data,
        //     &salt,
        // )        
    }
}
#[derive(StdHash, Debug)]
struct FuzzerMessage {
    caller: AccountId,
    callee: AccountId,
    endowment: Balance,
    input: Vec<u8>,
}

type FuzzerTrace = Vec<FuzzerCall>;
fn hash_trace(trace: &[FuzzerCall]) -> TraceHash {
    let mut hasher = DefaultHasher::new();
    trace.hash(&mut hasher);
    hasher.finish()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut fuzzer: RuntimeFuzzer =
        RuntimeFuzzer::new(PathBuf::from("./flipper/target/ink/flipper.contract"));

    loop {
    
    let r = fuzzer.run();
    println!("Result: {:?}", r);
    }

    return Ok(());
}

fn maint() -> Result<(), Box<dyn std::error::Error>> {
    // Get the number of available logical CPU cores
    let num_cpus = rayon::current_num_threads();
    println!("Number of CPU cores: {}", num_cpus);

    // Execute the main logic in parallel using Rayon
    (0..num_cpus).into_par_iter().for_each(|_| {
        if let Err(err) = execute_main_logic() {
            eprintln!("Error: {:?}", err);
        }
        println!("Thread {:?} finished", thread::current().id());
    });

    // let record = session.record().call_results();
    // for result in record {
    //     println!("{:?}\n", result);
    // }
    Ok(())
}

fn execute_main_logic() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::<MinimalRuntime>::new()?;

    // Load contract from file
    let contract_path = Path::new("./flipper/target/ink/flipper.contract");
    let contract = ContractBundle::load(contract_path).expect("Failed to load contract");

    session.deploy_bundle(contract.clone(), "new", &["true"], NO_SALT, NO_ENDOWMENT)?;

    let init_value: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
    println!("Initial value: {}", init_value);

    session.call("flip", NO_ARGS, NO_ENDOWMENT)??;

    let value: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
    println!("Value after flip: {}", value);

    // let record = session.record().call_results();
    // for result in record {
    //     println!("{:?}\n", result);
    // }

    Ok(())
}