use anyhow::Result;
use drink::{
    frame_support::{
        pallet_prelude::Encode,
        sp_runtime::traits::{Bounded, UniqueSaturatedInto},
        weights::constants::WEIGHT_PROOF_SIZE_PER_KB,
    },
    frame_system::offchain::{Account, SendSignedTransaction},
    pallet_contracts::Determinism,
    runtime::{AccountIdFor, HashFor, MinimalRuntime},
    session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
    BalanceOf, ContractBundle, Weight,
    sandbox::Snapshot,
};
use fastrand::Rng;
use hex;
use log::{debug, error, info, trace};
use parity_scale_codec::Compact as ScaleCompact;
use rayon::prelude::*;
use scale_info::{
    form::PortableForm, IntoPortable, PortableType, TypeDef, TypeDefArray,
    TypeDefBitSequence, TypeDefCompact, TypeDefComposite, TypeDefPrimitive,
    TypeDefSequence, TypeDefTuple, TypeDefVariant,
};
use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hash as StdHash, Hasher},
    path::{Path, PathBuf},
    thread,
};

//This defines all the configurable types based on the current runtime: MinimalRuntime
type Balance = BalanceOf<MinimalRuntime>;
type AccountId = AccountIdFor<MinimalRuntime>;
type Hash = HashFor<MinimalRuntime>;

struct RuntimeFuzzer {
    rng: RefCell<Rng>,
    contract_path: PathBuf,
    contract: ContractBundle,
    cache: HashMap<TraceHash, Snapshot>,
    //Settings
    pub potential_callers: Vec<AccountId>,
    ignore_pure_messages: bool,
    max_sequence_size: usize,
}

impl RuntimeFuzzer {
    fn new(contract_path: PathBuf) -> Self {
        let contract =
            ContractBundle::load(&contract_path).expect("Failed to load contract");
        Self {
            rng: RefCell::new(Rng::new()),
            contract_path,
            contract,
            cache: HashMap::new(),
            potential_callers: (0xA0..0xAF).map(|i| AccountId::from([i; 32])).collect(),
            ignore_pure_messages: true,
            max_sequence_size: 100,
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
        let size = self.rng.borrow_mut().usize(0..self.max_sequence_size);
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

        FuzzerDeploy {
            caller: self.generate_caller(),
            endowment: Default::default(),
            bytecode: self.contract.wasm.clone(),
            input: encoded,
            salt: Default::default(),
        }
    }

    // Generates a fuzzed message to be added in the trace
    fn generate_message(&self) -> FuzzerMessage {
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
        let mut encoded = selector.to_bytes().to_vec();

        let selected_args_type_defs = selectec_args_spec
            .iter()
            .map(|arg| self.get_typedef(arg.ty().ty().id).unwrap())
            .collect();

        let mut encoded_args = self.generate_arguments(selected_args_type_defs).unwrap();
        encoded.append(&mut encoded_args);

        FuzzerMessage {
            caller: self.generate_caller(),
            callee: AccountId::from([0; 32]),
            endowment: Default::default(),
            input: encoded,
        }
    }

    //This should generate a random account id from the set of potential callers
    fn generate_caller(&self) -> AccountId {
        match self.rng.borrow_mut().choice(&self.potential_callers) {
            Some(account) => account.clone(),
            None => AccountId::from([0; 32]),
        }
    }

    // Ge the initial state from the cache
    fn get_cached_state(&self, trace: &FuzzerTrace) -> (Option<&Snapshot>, usize) {
        let hash = hash_trace(&trace);
        for length in trace.len()..0 {
            let subtrace:&[FuzzerCall] = &trace[0..length];
            if let Some(result) = self.cache.get(&hash_trace(subtrace)){
                return (Some(result),  length);
            };
        }
        return (None, 0);
    }

    fn run_trace(&mut self, trace: &FuzzerTrace) -> Result<()> {
        let mut session = Session::<MinimalRuntime>::new()?;
        let (starting_state, offset) = self.get_cached_state(trace);
        //session.restore(starting_point);
        for (pos, call) in trace.iter().enumerate().skip(offset) {
            let result_ok = match call {
                FuzzerCall::Deploy(deploy) => {
                    session.deploy_contract(
                        deploy.caller,
                        deploy.endowment,
                        deploy.bytecode.clone(),
                        deploy.input.clone(),
                        deploy.salt.clone(),
                    )?;
                    todo!("Check if the deploiyment was successful and set the address somwhere");
                    true
                }
                FuzzerCall::Message(message) => {
                    session.call_contract(
                        message.caller,
                        message.callee,
                        message.endowment,
                        message.input.clone(),
                    )?;
                    todo!("Check if the message was successful");
                    true
                }
            };

            //Take a snapshot after every successful message dump others
            if result_ok {
                let snapshot = session.sandbox().take_snapshot();
                self.cache.insert(hash_trace(&trace[..=pos]), snapshot);
                // There has been progress!
                // Check all the properties in isolation using dry_run and log the result.
                todo!("Check the properties");
                //self.check_properties();
            }else{
                // bail out of the mainloop!
                return Ok(());
            }



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
    bytecode: Vec<u8>,
    input: Vec<u8>,
    salt: Vec<u8>,
}

#[derive(StdHash, Debug)]
struct FuzzerMessage {
    caller: AccountId,
    callee: AccountId,
    endowment: Balance,
    input: Vec<u8>,
}

// impl Hash for FuzzerCall {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.call_type.hash(state);
//         self.call_name.hash(state);
//         for arg in &self.call_args {
//             arg.hash(state);
//         }
//     }
// }

type FuzzerTrace = Vec<FuzzerCall>;
type TraceHash = u64;
fn hash_trace(trace: &[FuzzerCall]) -> u64 {
    let mut hasher = DefaultHasher::new();
    trace.hash(&mut hasher);
    hasher.finish()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fuzzer =
        RuntimeFuzzer::new(PathBuf::from("./flipper/target/ink/flipper.contract"));
    //println!("Generated constructor {:?}", fuzzer.generate_constructor());
    println!("Generated message {:?}", fuzzer.generate_message());
    println!("Generated message {:?}", fuzzer.generate_message());
    //println!("Generated message {:?}", fuzzer.generate_message());
    //return Ok(());



    // TODO! use a command line argument parsing lib. Check what ink/drink ppl uses
    let contract_path = Path::new("./flipper/target/ink/flipper.contract");
    let contract: ContractBundle =
        ContractBundle::load(contract_path).expect("Failed to load contract");

    let transcoder = contract.transcoder;
    let metadata = transcoder.metadata();
    //println!("type of metadata: {:?}", metadata);
    let constructors = metadata.spec().constructors();

    // for constructor in constructors {
    //     println!("Constructor: {:?}", constructor);
    // }

    let messages = metadata.spec().messages();
    // for message in messages {
    //     println!("Message: {:?}", message);
    // }

    let mut rng = Rng::new();

    let mut session = Session::<MinimalRuntime>::new()?;

    for caller in &fuzzer.potential_callers {
        let caller_balance = session.sandbox().free_balance(caller);
        println!("Caller balance before: {:?}", caller_balance);
        println!("Minting to caller: {:?}", caller);
        let res = session
            .sandbox()
            .mint_into(caller.to_owned(), Balance::max_value() / 1000);
        println!("Minting result: {:?}", res);
        let caller_balance = session.sandbox().free_balance(caller);
        println!("Caller balance after: {:?}", caller_balance);
        println!("\n");
    }

    let gas_limit = Weight::max_value() / 4;
    let constructor = fuzzer.generate_constructor();

    let balance_caller = session.sandbox().free_balance(&constructor.caller);
    println!("Caller balance: {:?}", balance_caller);
    println!("Caller address {:?}", constructor.caller);
    let contract_deploy = session.sandbox().deploy_contract(
        constructor.bytecode,
        0,
        constructor.input,
        constructor.salt,
        constructor.caller,
        gas_limit,
        Some(Balance::max_value() / 100000),
    );

    println!("Contract deploy: {:?}", contract_deploy);
    let deployment_address = contract_deploy.result.unwrap().account_id;

    // for i in  {
    //     let call = fuzzer.generate_message();
    // }
    let iterations = rng.u8(..10);
    println!("\n");
    println!("Iterations: {:?}", iterations);
    println!("\n");

    for i in 1..iterations {
        let message = fuzzer.generate_message();

        let caller_balance = session.sandbox().free_balance(&message.caller);
        println!("Caller {i} balance: {:?}", caller_balance);
        let result = session.sandbox().call_contract(
            deployment_address.clone(),
            message.endowment,
            message.input,
            message.caller,
            Weight::max_value(),
            Some(Balance::max_value() / 100000),
            Determinism::Enforced,
        );
        println!("Message result {i}: {:?}", result);
        println!("\n");
    }

    Ok(())

    //session.deploy_bundle(       , "new", &["true"], NO_SALT, NO_ENDOWMENT)?;

    //List types and methods
    // let methods_description = contract.contract.methods;

    // Run deploy(fuzzed_args)
    // N = FuzzedLen()
    // for i in 0..N:
    //     Functype = rng.choice(methods_description))
    //     for arg in functypes.args:
    //          match(arg.type){
    //              int => rng.int()
    //              bool => rng.bool()
    //              string => rng.string()
    //              address => rng.address()
    //              bytes => rng.bytes()
    //          }
    //     TAKE SNAPSHOT()

    //     FOR i in 0..M:
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
