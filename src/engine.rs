use crate::arguments::ArgumentsGenerator;
use crate::config::Config;
use crate::constants::Constants;
use crate::fuzzer::Fuzzer;
use crate::types::{AccountId, Balance, CodeHash, Hashing, TraceHash};

use anyhow::{anyhow, Ok, Result};
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
use log::{debug, info};
use rayon::prelude::*;
use scale_info::{form::PortableForm, TypeDef};
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash as StdHash, Hasher},
    path::{Path, PathBuf},
    thread,
};

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

#[derive(StdHash, Debug, Clone)]
struct Deploy {
    caller: AccountId,
    endowment: Balance,
    contract_bytes: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>,
    code_hash: CodeHash,
    address: AccountId,
}
impl Deploy {
    pub fn new(
        caller: AccountId,
        endowment: Balance,
        contract_bytes: Vec<u8>,
        data: Vec<u8>,
        salt: Vec<u8>,
    ) -> Self {
        let code_hash = Hashing::hash(&contract_bytes);
        let address = Self::calculate_address(&caller, &code_hash, &data, &salt);
        Self {
            caller,
            endowment,
            contract_bytes,
            data,
            salt,
            code_hash,
            address,
        }
    }
    fn calculate_code_hash(contract_bytes: &Vec<u8>) -> CodeHash {
        Hashing::hash(contract_bytes)
    }

    fn calculate_address(
        caller: &AccountId,
        code_hash: &CodeHash,
        data: &Vec<u8>,
        salt: &Vec<u8>,
    ) -> AccountId {
        <DefaultAddressGenerator as AddressGenerator<MinimalRuntime>>::contract_address(
            caller, code_hash, data, salt,
        )
    }
}

#[derive(StdHash, Debug, Clone)]
struct Message {
    caller: AccountId,
    callee: AccountId,
    endowment: Balance,
    input: Vec<u8>,
}

#[derive(Debug)]
struct Trace {
    deploy: Deploy,
    messages: Vec<Message>,
}

impl Trace {
    pub fn new(deploy: Deploy) -> Self {
        Self {
            deploy,
            messages: vec![],
        }
    }

    // This function should be used to push a new Message to the trace
    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn hash(&self) -> TraceHash {
        let mut hasher = DefaultHasher::new();
        self.deploy.hash(&mut hasher);
        self.messages.hash(&mut hasher);
        hasher.finish()
    }

    pub fn contract(&self) -> AccountId {
        self.deploy.address.clone()
    }

    pub fn last_message(&self) -> Result<&Message> {
        match self.messages.last() {
            Some(message) => Ok(message),
            None => Err(anyhow!("No messages in the trace")),
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

    pub fn new(contract_path: PathBuf, config: Config) -> Result<Self> {
        info!("Loading contract from {:?}", contract_path);
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
        fuzzer: &mut Fuzzer,
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
    ) -> Result<Deploy> {
        let (caller, encoded, endowment) = self.generate_basic(fuzzer, selector)?;
        Ok(Deploy::new(
            caller,
            endowment,
            self.contract.wasm.clone(),
            encoded,
            salt,
        ))
    }

    // Generates a fuzzed message to be appended in the trace
    fn generate_message(
        &self,
        fuzzer: &mut Fuzzer,
        message_selector: &[u8; 4],
        callee: &AccountId,
    ) -> Result<Message> {
        let (caller, encoded, endowment) =
            self.generate_basic(fuzzer, message_selector)?;
        Ok(Message {
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
        deploy: &Deploy,
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
        message: &Message,
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

    fn decode_message(&self, data: &Vec<u8>) -> Result<Value> {
        let decoded = self
            .contract
            .transcoder
            .decode_contract_message(&mut data.as_slice())
            .map_err(|e| anyhow::anyhow!("Error decoding message: {:?}", e))?;
        Ok(decoded)
    }

    fn decode_deploy(&self, data: &Vec<u8>) -> Result<Value> {
        let decoded = self
            .contract
            .transcoder
            .decode_contract_constructor(&mut data.as_slice())
            .map_err(|e| anyhow::anyhow!("Error decoding constructor: {:?}", e))?;
        Ok(decoded)
    }

    // Error if a property fail
    fn check_properties(
        &self,
        mut fuzzer: &mut Fuzzer,
        session: &mut Session<MinimalRuntime>,
        trace: &Trace,
    ) -> Result<()> {
        let contract_address = trace.contract();

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
                    match self.decode_deploy(&trace.deploy.data) {
                        Err(_e) => {
                            println!("Raw deploy: {:?}", &trace.deploy.data);
                        }
                        Result::Ok(x) => {
                            println!("Decoded deploy: {:?}", x);
                        }
                    }

                    // trace?
                    for message in trace
                        .messages
                        .iter()
                    {
                        match self.decode_message(&message.input) {
                            Err(_e) => {
                                println!("Raw message: {:?}", &message.input);
                            }
                            Result::Ok(x) => {
                                println!("Decoded message: {:?}", x);
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

    pub fn run_campaign(&mut self, max_iterations: usize) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut fuzzer = Fuzzer::new(0, self.config.constants.clone());
        let mut session: Session<MinimalRuntime> = Session::<MinimalRuntime>::new()?;
        // Execute the action: Initialize the state
        self.initialize_state(&mut session)?;

        for _ in 0..max_iterations {
            let r = self.run(&mut fuzzer, &mut session);
            println!("Result: {:?}", r);
        }
        println!("Elapsed time: {:?}", start_time.elapsed());
        Ok(())
    }
    // pub fn run_campaign_concurrent(&mut self, max_iterations: usize) -> Result<()> {
    //     let num_cpus = rayon::current_num_threads();
    //     println!("Number of CPU cores: {}", num_cpus);

    //     // Execute the main logic in parallel using Rayon
    //     (0..num_cpus).into_par_iter().for_each(|_| {
    //         if let Err(err) = self.run_campaign(1000) {
    //             eprintln!("Error: {:?}", err);
    //         }
    //         println!("Thread {:?} finished", thread::current().id());
    //     });
    //     Ok(())
    // }

    fn run(
        &mut self,
        mut fuzzer: &mut Fuzzer,
        session: &mut Session<MinimalRuntime>,
    ) -> Result<()> {
        debug!("Starting run");
        let mut current_state = None; // The current state not yet materialized in the session

        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //  Deploy the main contract to be fuzzed using a random constructor with fuzzed argumets
        let constructor_selector = fuzzer.choice(&self.constructors).unwrap();
        let constructor = self.generate_constructor(
            &mut fuzzer,
            constructor_selector,
            Default::default(),
        )?;

        // Start the trace with a deployment
        let mut trace = Trace::new(constructor);

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
                let result = self.execute_deploy(session, &trace.deploy)?;

                // Bail out if execution reverted
                if !result.flags.is_empty() {
                    anyhow::bail!("Execution reverted");
                };

                // If it did not revert
                self.check_properties(&mut fuzzer, session, &trace)?;

                // If the execution went ok then store the new state in the cache
                self.snapshot_cache
                    .insert(trace.hash(), session.sandbox().take_snapshot());
            }
        };

        let max_txs = self.config.max_number_of_transactions;
        let callee = trace.contract();
        for i in 0..max_txs {
            debug!("Tx: {}/{}", i, max_txs);

            let message_selector = fuzzer.choice(&self.messages).unwrap();
            let message =
                self.generate_message(&mut fuzzer, message_selector, &callee)?;
            trace.push(message);

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
                    let result = self.execute_message(session, trace.last_message()?)?;

                    // Bail out if execution reverted
                    if !result.flags.is_empty() {
                        anyhow::bail!("Execution reverted");
                    };

                    // If it did not revert
                    self.check_properties(&mut fuzzer, session, &trace)?;

                    // If the execution returned Ok(()) then store the new state in the cache
                    self.snapshot_cache
                        .insert(trace.hash(), session.sandbox().take_snapshot());
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //test that the hash of two FuzzTraces are equal
    #[test]
    fn test_hash_trace() {
        let caller = AccountId::new([0; 32]);
        let endowment = 0;
        let contract_bytes = vec![0, 1, 2, 3];
        let data = vec![4, 5, 6, 7];
        let salt = vec![8, 9, 10, 11];

        let deploy = Deploy::new(caller, endowment, contract_bytes, data, salt);
        let mut trace1 = Trace::new(deploy.clone());
        let mut trace2 = Trace::new(deploy);

        let message = Message {
            caller: AccountId::new([0; 32]),
            callee: AccountId::new([1; 32]),
            endowment: 0,
            input: vec![0, 1, 2, 3],
        };
        let message_identical = Message {
            caller: AccountId::new([0; 32]),
            callee: AccountId::new([1; 32]),
            endowment: 0,
            input: vec![0, 1, 2, 3],
        };

        trace1.push(message);
        trace2.push(message_identical);
        assert_eq!(&trace1.hash(), &trace2.hash());
    }
}
