mod arguments;
mod config;
mod constants;
mod fuzzer;
mod types;

use anyhow::{anyhow, Ok, Result};
use arguments::ArgumentsGenerator;
use constants::Constants;
use drink::{
    contracts_api,
    frame_support::sp_runtime::traits::Hash as HashTrait,
    pallet_contracts::{
        AddressGenerator, DefaultAddressGenerator, Determinism, ExecReturnValue,
    },
    runtime::MinimalRuntime,
    sandbox::Snapshot,
    session::{contract_transcode::Value, Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
    ContractBundle,
};

use clap::{Args, Parser, Subcommand};
use config::Config;
use fuzzer::Fuzzer;
use log::{debug, info};
use rayon::prelude::*;
use scale_info::{form::PortableForm, TypeDef};
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash as StdHash, Hasher},
    path::{Path, PathBuf},
    thread,
};
use types::{AccountId, Balance, CodeHash, Hashing, TraceHash};

// Our own copy of method information. The selector is used as the key in the hashmap
struct MethodInfo {
    arguments: Vec<TypeDef<PortableForm>>,
    mutates: bool,
    payable: bool,
    constructor: bool,
}
impl MethodInfo {
    fn new(
        arguments: Vec<TypeDef<PortableForm>>,
        mutates: bool,
        payable: bool,
        constructor: bool,
    ) -> Self {
        Self {
            arguments,
            mutates,
            payable,
            constructor,
        }
    }
}

pub struct Engine {
    // Contract Info
    contract_path: PathBuf,
    contract: ContractBundle,

    // Rapid access to function info
    method_info: HashMap<[u8; 4], MethodInfo>,
    constructors: HashSet<[u8; 4]>,
    messages: HashSet<[u8; 4]>,
    properties: HashSet<[u8; 4]>,

    // Cache
    snapshot_cache: HashMap<TraceHash, Snapshot>,

    // Settings
    config: Config,
}

impl Engine {
    //This should generate a random account id from the set of potential callers
    fn generate_caller(&self, fuzzer: &mut Fuzzer) -> AccountId {
        fuzzer
            .choice(&self.config.accounts)
            .expect("You need to configure some potential callers")
            .clone()
    }

    fn generate_endowment(&self, fuzzer: &mut Fuzzer, _caller: &AccountId) -> Balance {
        // TODO! This should be a sensible value related to the balance of the caller
        // endowment should be in the range [0, balanceOf(caller) - existentialDeposit)
        let max_endowment = self.config.budget;
        *fuzzer
            .choice([0, 1, max_endowment / 2, max_endowment - 1, max_endowment].iter())
            .unwrap() as Balance
    }

    fn extract_method_info(&mut self) -> Result<()> {
        let ink = self.contract.transcoder.metadata();
        let registry = ink.registry();

        for spec in ink.spec().constructors().iter() {
            let selector: [u8; 4] = spec
                .selector()
                .to_bytes()
                .try_into()
                .expect("Selector Must be 4 bytes long");

            let mut arguments = vec![];
            for arg in spec.args() {
                let arg = &registry
                    .resolve(arg.ty().ty().id)
                    .ok_or(anyhow!("Cannot resolve {:?}", arg))?
                    .type_def;
                arguments.push(arg.clone());
            }
            let method_info = MethodInfo::new(arguments, true, spec.payable, true);
            self.method_info.insert(selector, method_info);
            self.constructors.insert(selector);
        }
        for spec in ink.spec().messages().iter() {
            let selector: [u8; 4] = spec
                .selector()
                .to_bytes()
                .try_into()
                .expect("Selector Must be 4 bytes long");
            let mut arguments = vec![];
            for arg in spec.args() {
                let arg = &registry
                    .resolve(arg.ty().ty().id)
                    .ok_or(anyhow!("Cannot resolve {:?}", arg))?
                    .type_def;
                arguments.push(arg.clone());
            }
            let method_info =
                MethodInfo::new(arguments, spec.mutates(), spec.payable(), false);
            self.method_info.insert(selector, method_info);
            if self.is_property(spec.label()) {
                self.properties.insert(selector);
            }
            //TODO: configure if we must use messages that are marked as non mutating
            if !self.config.only_mutable || spec.mutates() {
                self.messages.insert(selector);
            }
        }
        Ok(())
    }

    fn new(contract_path: PathBuf, config: Config) -> Result<Self> {
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
        println!("Loading contract from {:?}", contract_path);
        let contract = ContractBundle::load(&contract_path)?;

        //TODO: fix callers
        let _default_callers: Vec<AccountId> = vec![AccountId::new([41u8; 32])];
        let mut runtime_fuzzer = Self {
            // Contract Info
            contract_path,
            contract,

            // Rapid access to function info
            method_info: HashMap::new(),
            constructors: HashSet::new(),
            messages: HashSet::new(),
            properties: HashSet::new(),

            // Cache
            snapshot_cache: HashMap::new(),

            // Settings
            config,
        };
        runtime_fuzzer.extract_method_info()?;
        Ok(runtime_fuzzer)
    }

    fn generate_basic(
        &self,
        mut fuzzer: &mut Fuzzer,
        selector: &[u8; 4],
    ) -> Result<(AccountId, Vec<u8>, Balance)> {
        let method_info = match self.method_info.get(selector) {
            Some(method_info) => method_info,
            None => anyhow::bail!("No arguments for the selected constructor"),
        };
        let is_payable = method_info.payable;
        let generator = ArgumentsGenerator::new(
            self.contract.transcoder.metadata().registry(),
            &method_info.arguments,
        );
        let mut encoded_arguments = generator.generate(fuzzer)?;

        let caller = self.generate_caller(fuzzer);
        // Send endowment only if the constructor is marked as payable
        let endowment = if is_payable {
            self.generate_endowment(fuzzer, &caller)
        } else {
            0
        };

        // Build the encoded calldata. Starting by the selector.
        let mut encoded = selector.to_vec();
        encoded.append(&mut encoded_arguments);

        Ok((caller, encoded, endowment))
    }

    // Generates a fuzzed constructor to be prepended in the trace
    fn generate_constructor(
        &self,
        fuzzer: &mut Fuzzer,
        selector: &[u8; 4],
        salt: Vec<u8>,
    ) -> Result<FuzzerDeploy> {
        let (caller, encoded, endowment) = self.generate_basic(fuzzer, selector)?;
        Ok(FuzzerDeploy {
            caller,
            endowment,
            contract_bytes: self.contract.wasm.clone(),
            data: encoded,
            salt,
        })
    }

    // Generates a fuzzed message to be appended in the trace
    fn generate_message(
        &self,
        fuzzer: &mut Fuzzer,
        message_selector: &[u8; 4],
        callee: &AccountId,
    ) -> Result<FuzzerMessage> {
        let (caller, encoded, endowment) =
            self.generate_basic(fuzzer, message_selector)?;
        Ok(FuzzerMessage {
            caller,
            callee: callee.clone(),
            endowment,
            input: encoded,
        })
    }

    // Defines which method names will be considered to be a property
    fn is_property(&self, function_name: &str) -> bool {
        function_name.starts_with(self.config.property_prefix.as_str())
    }

    fn initialize_state(&self, session: &mut Session<MinimalRuntime>) -> Result<()> {
        debug!("Setting initial state. Give initial budget to caller addresses.");
        // Assigning initial budget to caller addresses
        let sandbox = session.sandbox();
        for account in &self.config.accounts {
            debug!("  Mint {} to {}", self.config.budget, account);
            sandbox
                .mint_into(account.clone(), self.config.budget)
                .map_err(|e| anyhow::anyhow!("Error minting into account: {:?}", e))?;
        }
        Ok(())
    }

    // Exceutes the call on the given session
    fn execute_deploy(
        &self,
        session: &mut Session<MinimalRuntime>,
        deploy: &FuzzerDeploy,
    ) -> Result<ExecReturnValue> {
        info!("Deploying contract with data {:?}", deploy);
        let deployment_result = session.sandbox().deploy_contract(
            deploy.contract_bytes.clone(),
            0,
            deploy.data.clone(),
            deploy.salt.clone(),
            deploy.caller.clone(),
            self.config.gas_limit,
            None,
        );
        let parsed_deployment = deployment_result
            .result
            .map_err(|e| anyhow::anyhow!("Error executing deploy: {:?}", e))?;
        let result = parsed_deployment.result;
        debug!("Deploy Result: {:?}", result);
        Ok(result)
    }

    // Exceutes the message on the given session
    fn execute_message(
        &self,
        session: &mut Session<MinimalRuntime>,
        message: &FuzzerMessage,
    ) -> Result<ExecReturnValue> {
        //TODO: This result has to be checked for reverts. In the flags field we can find the revert flag
        info!("Sending message with data {:?}", message);
        let result = session
            .sandbox()
            .call_contract(
                message.callee.clone(),
                message.endowment,
                message.input.clone(),
                message.caller.clone(),
                self.config.gas_limit,
                None,
                Determinism::Enforced,
            )
            .result
            .map_err(|e| anyhow::anyhow!("Error executing message: {:?}", e))?;
        debug!("Result: {:?}", result);
        Ok(result)
    }

    // Exceutes the call on the given session
    fn execute_call(
        &self,
        session: &mut Session<MinimalRuntime>,
        call: &FuzzerCall,
    ) -> Result<ExecReturnValue> {
        Ok(match call {
            FuzzerCall::Message(message) => self.execute_message(session, message)?,
            FuzzerCall::Deploy(deploy) => self.execute_deploy(session, deploy)?,
        })
    }

    fn decode_call(&self, call: &FuzzerCall) -> Result<Value> {
        match call {
            FuzzerCall::Message(message) => {
                let decoded = self
                    .contract
                    .transcoder
                    .decode_contract_message(&mut message.input.as_slice())
                    .map_err(|e| anyhow::anyhow!("Error decoding message: {:?}", e))?;
                Ok(decoded)
            }
            FuzzerCall::Deploy(deploy) => {
                let decoded = self
                    .contract
                    .transcoder
                    .decode_contract_constructor(&mut deploy.data.as_slice())
                    .map_err(|e| {
                        anyhow::anyhow!("Error decoding constructor: {:?}", e)
                    })?;
                Ok(decoded)
            }
        }
    }

    // Error if a property fail
    fn check_properties(
        &self,
        mut fuzzer: &mut Fuzzer,
        session: &mut Session<MinimalRuntime>,
        trace: &FuzzerTrace,
    ) -> Result<()> {
        let contract_address = trace.contract()?;

        // Properties should not affect the state
        // We save a snapshot before the properties so we can restore it later. Effectively a dry-run
        let checkpoint = session.sandbox().take_snapshot();
        let properties = self.properties.clone();
        for property in properties.iter() {
            let arguments_length = self
                .method_info
                .get(property)
                .map_or(0usize, |method_info| method_info.arguments.len());

            let max_rounds = if arguments_length == 0 {
                // No arguments execute the property only once
                1usize
            } else {
                // Multiple arguments execute the property multiple times
                self.config.fuzz_property_max_rounds ^ arguments_length
            };
            for _round in 0..max_rounds {
                let property_message =
                    self.generate_message(fuzzer, property, &contract_address)?;
                let result = self.execute_message(session, &property_message);
                let failed = match result {
                    Err(e) => {
                        debug!("Property check failed: {:?}", e);
                        true
                    }
                    Result::Ok(result) => result.data == vec![0, 0],
                };
                session.sandbox().restore_snapshot(checkpoint.clone());

                // Property must return 0, 1 always otherwise it is a broken property
                if failed {
                    println!("Property check failed");
                    // trace?
                    for call in trace
                        .messages
                        .iter()
                        .chain(&[FuzzerCall::Message(property_message)])
                    {
                        match self.decode_call(call) {
                            Err(_e) => {
                                println!("Raw call: {:?}", call.data());
                            }
                            Result::Ok(x) => {
                                println!("Decoded call: {:?}", x);
                            }
                        }
                    }
                    //println!("Property failed at trace {:?}", trace);
                    anyhow::bail!("Property check failed");
                }
            }
        }

        Ok(())
    }

    fn run(
        &mut self,
        mut fuzzer: &mut Fuzzer,
        session: &mut Session<MinimalRuntime>,
    ) -> Result<()> {
        debug!("Starting run");
        let mut trace = FuzzerTrace::new(); // The execution trace
        let mut current_state = None; // The current state not yet materialized in the session

        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // Initialize the state: Assigning initial budget to caller addresses

        // CACHE: Check if the cache knows how to initialize the state
        match self.snapshot_cache.get(&trace.hash()) {
            Some(snapshot) => {
                // The initial state is already in the cache.
                current_state = Some(snapshot);
                debug!("Cahe HIT: Initialization is taken from the cache");
            }
            None => {
                debug!("Cahe MISS: We need to run the initialization at least once");

                // It should not be a pending current state
                assert!(current_state.is_none());

                // Execute the action: Initialize the state
                self.initialize_state(session)?;

                // If the initialization went ok then save the result in the cache so we do not need to re do
                self.snapshot_cache.insert(
                    hash_trace(&trace.messages),
                    session.sandbox().take_snapshot(),
                );

                // No pending current state.
                current_state = None;
            }
        };

        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //  Deploy the main contract to be fuzzed using a random constructor with fuzzed argumets

        let constructor_selector = fuzzer.choice(&self.constructors).unwrap();
        let constructor = self.generate_constructor(
            &mut fuzzer,
            constructor_selector,
            Default::default(),
        )?;

        // Add the deployment to the trace.
        trace.push(FuzzerCall::Deploy(constructor))?;

        // CACHE: Check we happened to choose the same constructor as a previous run
        match self.snapshot_cache.get(&trace.hash()) {
            Some(snapshot) => {
                debug!("Cahe HIT: Same constructor was choosen and executed before, reloading state from cache");
                // The trace was already in the cache set current pending state
                current_state = Some(snapshot);
            }
            None => {
                debug!("Cahe MISS: The choosen constructor was never executed before. Executing it.");
                // The trace was not in the cache, apply the previous state if any
                if let Some(snapshot) = current_state {
                    debug!("The current state is not yet materialized in the session, restoring current state.");
                    session.sandbox().restore_snapshot(snapshot.clone());
                };
                // The current state is already in the session. Next step needs not to load it from the `current_state`
                current_state = None;

                // Execute the action
                let result = self.execute_call(session, trace.last()?)?;

                // Bail out if execution reverted
                if !result.flags.is_empty() {
                    anyhow::bail!("Execution reverted");
                };

                // If it did not revert
                self.check_properties(&mut fuzzer, session, &trace)?;

                // If the execution went ok then store the new state in the cache
                self.snapshot_cache.insert(
                    hash_trace(&trace.messages),
                    session.sandbox().take_snapshot(),
                );
            }
        };

        let max_txs = self.config.max_number_of_transactions;
        let contract_address = trace.contract()?;
        for i in 0..max_txs {
            debug!("Tx: {}/{}", i, max_txs);

            let message = {
                let message_selector = fuzzer.choice(&self.messages).unwrap();
                let message = self.generate_message(
                    &mut fuzzer,
                    message_selector,
                    &contract_address,
                )?;
                message.clone()
            };
            trace.push(FuzzerCall::Message(message))?;

            // CACHE: Check we happened to choose the same trace prolog as a previous run
            match self.snapshot_cache.get(&trace.hash()) {
                Some(snapshot) => {
                    debug!("Cahe HIT: at iteration {}, Same trace prolog was choosen and executed before before, reloading state from cache", i);
                    // The trace is already in the cache set current state
                    current_state = Some(snapshot);
                }
                None => {
                    debug!("Cahe MISS: Same trace prolog was never executed before");

                    // The trace was not in the cache, apply the previous state if any
                    if let Some(snapshot) = current_state {
                        debug!("At iteration {}, the current state is not yet materialized in the session, restoring current state.", i);
                        session.sandbox().restore_snapshot(snapshot.clone());
                    }
                    // The current state is already in the session. Next step needs not to load it from the `current_state`
                    current_state = None;

                    // Execute the action
                    let result = self.execute_call(session, trace.last()?)?;

                    // Bail out if execution reverted
                    if !result.flags.is_empty() {
                        anyhow::bail!("Execution reverted");
                    };

                    // If it did not revert
                    self.check_properties(&mut fuzzer, session, &trace)?;

                    // If the execution returned Ok(()) then store the new state in the cache
                    self.snapshot_cache.insert(
                        hash_trace(&trace.messages),
                        session.sandbox().take_snapshot(),
                    );
                }
            };
        }
        Ok(())
    }
}

#[derive(StdHash, Debug)]
enum FuzzerCall {
    Deploy(FuzzerDeploy),
    Message(FuzzerMessage),
}
impl FuzzerCall {
    fn data(&self) -> &Vec<u8> {
        match self {
            FuzzerCall::Deploy(deploy) => &deploy.data,
            FuzzerCall::Message(message) => &message.input,
        }
    }

    fn caller(&self) -> &AccountId {
        match self {
            FuzzerCall::Deploy(deploy) => &deploy.caller,
            FuzzerCall::Message(message) => &message.caller,
        }
    }

    fn endowment(&self) -> &Balance {
        match self {
            FuzzerCall::Deploy(deploy) => &deploy.endowment,
            FuzzerCall::Message(message) => &message.endowment,
        }
    }

    fn callee(&self) -> AccountId {
        match self {
            FuzzerCall::Deploy(deploy) => deploy.calculate_address(),
            FuzzerCall::Message(message) => message.callee.clone(),
        }
    }
}

#[derive(StdHash, Debug, Clone)]
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
        let caller_address = &self.caller;
        let code_hash: CodeHash = self.calculate_code_hash();
        let input_data = &self.data;
        let salt = &self.salt;

        <DefaultAddressGenerator as AddressGenerator<MinimalRuntime>>::contract_address(
            caller_address,
            &code_hash,
            input_data,
            salt,
        )
    }
}
#[derive(StdHash, Debug, Clone)]
struct FuzzerMessage {
    caller: AccountId,
    callee: AccountId,
    endowment: Balance,
    input: Vec<u8>,
}

#[derive(Debug)]
struct FuzzerTrace {
    messages: Vec<FuzzerCall>,
    contract: Option<AccountId>,
}

impl FuzzerTrace {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            contract: None,
        }
    }
    pub fn push(&mut self, message: FuzzerCall) -> Result<()> {
        if self.messages.is_empty() {
            //Force That the first FuzzerCall to be a deployment
            if let FuzzerCall::Message(_m) = message {
                anyhow::bail!("First call must be a deployment")
            }
        }
        self.messages.push(message);
        if self.messages.len() == 1 {
            self.contract = match self.messages.first() {
                Some(deploy) => match deploy {
                    FuzzerCall::Deploy(deploy) => Some(deploy.calculate_address()),
                    FuzzerCall::Message(_) => {
                        anyhow::bail!("Trace must start with a deploy")
                    }
                },
                None => anyhow::bail!("No deploys to execute"),
            };
        }

        Ok(())
    }

    pub fn hash(&self) -> TraceHash {
        let mut hasher = DefaultHasher::new();
        self.messages.hash(&mut hasher);
        hasher.finish()
    }

    pub fn contract(&self) -> Result<AccountId> {
        match &self.contract {
            Some(contract) => Ok(contract.to_owned()),
            None => anyhow::bail!("Contract not set yet."),
        }
    }

    pub fn last(&self) -> Result<&FuzzerCall> {
        match self.messages.last() {
            Some(call) => Ok(call),
            None => anyhow::bail!("No calls to execute"),
        }
    }
}

fn hash_trace(trace: &[FuzzerCall]) -> TraceHash {
    let mut hasher = DefaultHasher::new();
    trace.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Parser)]
struct Cli {
    /// input file
    #[clap(index = 1)]
    pub contract: PathBuf,

    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    // This initializes the logging. The code uses debug! info! trace! and error! macros
    // You can enable the output via the environment variable RUST_LOG
    env_logger::init();

    // Parse the command line arguments
    let cli = Cli::parse();

    // Used for developement when the Config format is changed
    //Config::default().to_yaml_file(&cli.config)?;
    let config = Config::from_yaml_file(&cli.config).expect("failed to parse yaml file");
    let contract_path = cli.contract;

    let mut runtime = Engine::new(contract_path, config)?;
    let mut session: Session<MinimalRuntime> = Session::<MinimalRuntime>::new()?;
    let mut fuzzer = Fuzzer::new(0, runtime.config.constants.clone());

    let start_time = std::time::Instant::now();
    for _ in 0..1000 {
        let r = runtime.run(&mut fuzzer, &mut session);
        println!("Result: {:?}", r);
    }
    println!("Elapsed time: {:?}", start_time.elapsed());
    Ok(())
}

// fn mainy() -> Result<()> {
//     // This initializes the logging. The code uses debug! info! trace! and error! macros
//     // You can enable the output via the environment variable RUST_LOG
//     env_logger::init();
//     let mut fuzzer = RuntimeFuzzer::new(
//         PathBuf::from("./test-contracts/ityfuzz/target/ink/ityfuzz.contract"),
//         Constants::default(),
//     )?;

//     let mut session: Session<MinimalRuntime> = Session::<MinimalRuntime>::new()?;

//     let start_time = std::time::Instant::now();
//     for _ in 0..1000 {
//         let r = fuzzer.run(&mut session);
//         println!("Result: {:?}", r);
//     }
//     println!("Elapsed time: {:?}", start_time.elapsed());
//     Ok(())
// }

fn maint() -> Result<()> {
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

fn execute_main_logic() -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    //test that the hash of two FuzzTraces are equal
    #[test]
    fn test_hash_trace() {
        let mut trace1 = FuzzerTrace::new();
        let mut trace2 = FuzzerTrace::new();
        let deploy = FuzzerDeploy {
            caller: AccountId::new([0; 32]),
            endowment: 0,
            contract_bytes: vec![0, 1, 2, 3],
            data: vec![4, 5, 6, 7],
            salt: vec![8, 9, 10, 11],
        };
        let message = FuzzerMessage {
            caller: AccountId::new([0; 32]),
            callee: AccountId::new([1; 32]),
            endowment: 0,
            input: vec![0, 1, 2, 3],
        };
        let message_identical = FuzzerMessage {
            caller: AccountId::new([0; 32]),
            callee: AccountId::new([1; 32]),
            endowment: 0,
            input: vec![0, 1, 2, 3],
        };

        trace1.push(FuzzerCall::Deploy(deploy.clone())).unwrap();
        trace1.push(FuzzerCall::Message(message)).unwrap();
        trace2.push(FuzzerCall::Deploy(deploy)).unwrap();
        trace2.push(FuzzerCall::Message(message_identical)).unwrap();
        assert_eq!(hash_trace(&trace1.messages), hash_trace(&trace2.messages));
    }
}
