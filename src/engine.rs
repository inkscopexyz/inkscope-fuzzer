use crate::{
    config::Config,
    contract_bundle::ContractBundle,
    fuzzer::Fuzzer,
    generator::Generator,
    types::{
        AccountId,
        Balance,
        CodeHash,
        Hashing,
        TraceHash,
    },
};

use anyhow::{
    anyhow,
    Ok,
    Result,
};
use contract_transcode::Value;
use ink_sandbox::{
    api::{
        balance_api::BalanceAPI,
        contracts_api::ContractAPI,
    },
    frame_support::sp_runtime::traits::Hash,
    macros::DefaultSandboxRuntime,
    pallet_contracts::{
        AddressGenerator,
        DefaultAddressGenerator,
        Determinism,
        ExecReturnValue,
    },
    DefaultSandbox,
    DispatchError,
    Sandbox,
    Snapshot,
};

use log::{
    debug,
    info,
};
use parity_scale_codec::Encode;
use scale_info::{
    form::PortableForm,
    TypeDef,
};
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    hash::{
        DefaultHasher,
        Hash as StdHash,
        Hasher,
    },
    path::PathBuf,
};

pub struct CampaignResult {
    pub failed_traces: Vec<FailedTrace>,
}

// When a failed trace is found, there are 3 possible reasons:
// 1. The contract panicked during the execution of the deploy, in this case the
//    failed_properties will be empty, and the trace will contain only the deploy that
//    failed.
// 2. The contract panicked during the execution of a message because of a
//    ContractTrapped, in this case the failed_properties will be empty, and the trace
//    will contain the deploy and all the messages that were executed, being the last
//    message the one that failed.
// 3. One or more properties defined by the developer did not hold, in this case the
//    failed_properties vec will contain the ones that broke, and the trace will contain
//    the deploy and all the messages that were executed before the property failed.
pub struct FailedTrace {
    pub trace: Trace,
    pub failed_properties: Vec<Message>,
}

// Our own copy of method information. The selector is used as the key in the hashmap
struct MethodInfo {
    arguments: Vec<TypeDef<PortableForm>>,
    #[allow(dead_code)]
    mutates: bool,
    payable: bool,
    #[allow(dead_code)]
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
pub struct Deploy {
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

    fn calculate_address(
        caller: &AccountId,
        code_hash: &CodeHash,
        data: &[u8],
        salt: &[u8],
    ) -> AccountId {
        <DefaultAddressGenerator as AddressGenerator<DefaultSandboxRuntime>>::contract_address(
            caller, code_hash, data, salt,
        )
    }
}

#[derive(StdHash, Debug, Clone)]
pub struct Message {
    caller: AccountId,
    callee: AccountId,
    endowment: Balance,
    pub input: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Trace {
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

pub enum MessageOrDeployResult {
    Trapped,
    Reverted,
    Success(Vec<u8>),
    Unhandled(DispatchError),
}
impl From<Result<ExecReturnValue, DispatchError>> for MessageOrDeployResult {
    fn from(val: Result<ExecReturnValue, DispatchError>) -> Self {
        match val {
            Err(e) => {
                // If the deployment panics, we consider it failed
                match e {
                    DispatchError::Module(module_error)
                        if (module_error.message == Some("ContractTrapped")) =>
                    {
                        MessageOrDeployResult::Trapped
                    }
                    _ => MessageOrDeployResult::Unhandled(e),
                }
            }
            // Return if execution reverted in constructor
            Result::Ok(res) => {
                if !res.flags.is_empty() {
                    MessageOrDeployResult::Reverted
                } else {
                    MessageOrDeployResult::Success(res.data)
                }
            }
        }
    }
}

pub struct Engine {
    // Contract Info
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
    // This should generate a random account id from the set of potential callers
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
            // TODO: configure if we must use messages that are marked as non mutating
            if !self.config.only_mutable || spec.mutates() {
                self.messages.insert(selector);
            }
        }
        Ok(())
    }

    pub fn new(contract_path: PathBuf, config: Config) -> Result<Self> {
        info!("Loading contract from {:?}", contract_path);
        let contract = ContractBundle::load(contract_path)?;

        // TODO: fix callers
        let _default_callers: Vec<AccountId> = vec![AccountId::new([41u8; 32])];
        let mut engine = Self {
            // Contract Info
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
        engine.extract_method_info()?;
        Ok(engine)
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
        let generator = Generator::new(self.contract.transcoder.metadata().registry());
        let mut encoded_arguments = generator.generate(fuzzer, &method_info.arguments)?;

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

    fn initialize_state(&self, sandbox: &mut DefaultSandbox) -> Result<()> {
        debug!("Setting initial state. Give initial budget to caller addresses.");
        // Assigning initial budget to caller addresses
        for account in &self.config.accounts {
            debug!("  Mint {} to {}", self.config.budget, account);
            sandbox
                .mint_into(account, self.config.budget)
                .map_err(|e| anyhow::anyhow!("Error minting into account: {:?}", e))?;
        }
        Ok(())
    }

    // Exceutes the call on the given sandbox
    fn execute_deploy(
        &self,
        sandbox: &mut DefaultSandbox,
        deploy: &Deploy,
    ) -> MessageOrDeployResult {
        info!("Deploying contract with data {:?}", deploy);
        let deployment_result = sandbox.deploy_contract(
            deploy.contract_bytes.clone(),
            0,
            deploy.data.clone(),
            deploy.salt.clone(),
            deploy.caller.clone(),
            self.config.gas_limit,
            None,
        );
        deployment_result.result.map(|res| res.result).into()
    }

    // Exceutes the message on the given sandbox
    fn execute_message(
        &self,
        sandbox: &mut DefaultSandbox,
        message: &Message,
    ) -> MessageOrDeployResult {
        info!("Sending message with data {:?}", message);
        sandbox
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
            .into()
    }

    // Error if a property fail
    fn check_properties(
        &self,
        fuzzer: &mut Fuzzer,
        sandbox: &mut DefaultSandbox,
        trace: &Trace,
    ) -> Result<Vec<Message>> {
        let mut failed_properties = vec![];
        let contract_address = trace.contract();

        // Properties should not affect the state
        // We save a snapshot before the properties so we can restore it later.
        // Effectively a dry-run
        let checkpoint = sandbox.take_snapshot();
        let properties = self.properties.clone();

        // For each property, we will only try to break it once. If we find an argument
        // that makes it return false, we will move on to the next property
        // without looking for more examples. We finish the search on the first example
        // that breaks it.
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

            // If the property has arguments, fuzz them
            for _round in 0..max_rounds {
                let property_message =
                    self.generate_message(fuzzer, property, &contract_address)?;

                let result = self.execute_message(sandbox, &property_message);

                // We restore the state to the snapshot taken before executing the
                // property
                sandbox.restore_snapshot(checkpoint.clone());

                match result {
                    MessageOrDeployResult::Unhandled(e) => {
                        // If the error is not a ContractTrapped, we panic because
                        // is not an expected behavior
                        panic!("Error: {:?}", e);
                    }
                    MessageOrDeployResult::Trapped | MessageOrDeployResult::Reverted => {
                        // If the property reverts or panics, we also consider it failed
                        failed_properties.push(property_message);
                        break;
                    }
                    MessageOrDeployResult::Success(data) => {
                        // A property is considered failed if the result of calling the
                        // property is Ok(false)
                        if data == std::result::Result::<bool, ()>::Ok(false).encode() {
                            failed_properties.push(property_message);
                            break;
                        }
                    }
                }
            }
        }

        Ok(failed_properties)
    }

    pub fn run_campaign(&mut self) -> Result<CampaignResult> {
        let max_iterations = self.config.max_rounds;
        let fail_fast = self.config.fail_fast;

        let start_time = std::time::Instant::now();
        let mut failed_traces = vec![];
        let mut fuzzer = Fuzzer::new(0, self.config.constants.clone());

        for _ in 0..max_iterations {
            if let Some(failed_trace) = self.run(&mut fuzzer)? {
                // Fail fast if a property fails
                failed_traces.push(failed_trace);
                if fail_fast {
                    break;
                }
            }
        }

        println!("Elapsed time: {:?}", start_time.elapsed());
        Ok(CampaignResult { failed_traces })
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
    fn run(&mut self, fuzzer: &mut Fuzzer) -> Result<Option<FailedTrace>> {
        debug!("Starting run");

        /// Hardcoded empty trace hash
        const EMPTY_TRACE_HASH: u64 = 0;

        let mut sandbox = DefaultSandbox::default();

        // Check if the initial state is already in the cache
        let mut current_state = match self.snapshot_cache.get(&EMPTY_TRACE_HASH) {
            Some(init_snapshot) => {
                sandbox.restore_snapshot(init_snapshot.clone());
                Some(init_snapshot)
            }
            _ => {
                self.initialize_state(&mut sandbox)?;
                self.snapshot_cache
                    .insert(EMPTY_TRACE_HASH, sandbox.take_snapshot());
                None
            }
        };

        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //  Deploy the main contract to be fuzzed using a random constructor with fuzzed
        // argumets
        let constructor_selector = fuzzer.choice(&self.constructors).unwrap();
        let constructor =
            self.generate_constructor(fuzzer, constructor_selector, Default::default())?;

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
                    debug!("The current state is not yet materialized in the sandbox, restoring current state.");
                    sandbox.restore_snapshot(snapshot.clone());
                };
                // The current state is already in the sandbox. Next step needs not to
                // load it from the `current_state`
                current_state = None;

                // Execute the action
                let result = self.execute_deploy(&mut sandbox, &trace.deploy);

                match result {
                    MessageOrDeployResult::Unhandled(e) => {
                        // If the error is not a ContractTrapped, we panic because
                        // is not an expected behavior
                        panic!("Error: {:?}", e);
                    }
                    MessageOrDeployResult::Trapped => {
                        return Ok(Some(FailedTrace {
                            trace,
                            failed_properties: vec![],
                        }));
                    }
                    MessageOrDeployResult::Reverted => {
                        // If the deployment reverts we just ignore this execution and
                        // start a new trace again
                        return Ok(None);
                    }
                    MessageOrDeployResult::Success(_) => {
                        // If it did not revert or panic we check the properties
                        let failed_properties =
                            self.check_properties(fuzzer, &mut sandbox, &trace)?;

                        if !failed_properties.is_empty() {
                            return Ok(Some(FailedTrace {
                                trace,
                                failed_properties,
                            }));
                        }

                        // If the execution went ok then store the new state in the cache
                        self.snapshot_cache
                            .insert(trace.hash(), sandbox.take_snapshot());
                    }
                }
            }
        };

        let max_txs = self.config.max_number_of_transactions;
        let callee = trace.contract();
        for i in 0..max_txs {
            debug!("Tx: {}/{}", i, max_txs);

            let message_selector = fuzzer.choice(&self.messages).unwrap();
            let message = self.generate_message(fuzzer, message_selector, &callee)?;
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
                        debug!("At iteration {}, the current state is not yet materialized in the sandbox, restoring current state.", i);
                        sandbox.restore_snapshot(snapshot.clone());
                    }
                    // The current state is already in the sandbox. Next step needs not to
                    // load it from the `current_state`
                    current_state = None;

                    // Execute the action
                    let result =
                        self.execute_message(&mut sandbox, trace.last_message()?);

                    match result {
                        MessageOrDeployResult::Unhandled(e) => {
                            // If the error is not a ContractTrapped, we panic because
                            // is not an expected behavior
                            panic!("Error: {:?}", e);
                        }
                        MessageOrDeployResult::Trapped => {
                            return Ok(Some(FailedTrace {
                                trace,
                                failed_properties: vec![], /* TODO: Add comment
                                                            * explaining this case */
                            }));
                        }
                        MessageOrDeployResult::Reverted => {
                            trace.messages.pop();
                            continue;
                        }
                        MessageOrDeployResult::Success(_) => {
                            // If it did not revert or panic we check the properties
                            let failed_properties =
                                self.check_properties(fuzzer, &mut sandbox, &trace)?;

                            // Once we find that at least one property failed, we stop
                            if !failed_properties.is_empty() {
                                return Ok(Some(FailedTrace {
                                    trace,
                                    failed_properties,
                                }));
                            }

                            // If the execution returned Ok(()) then store the new state
                            // in the cache
                            self.snapshot_cache
                                .insert(trace.hash(), sandbox.take_snapshot());
                        }
                    }
                }
            };
        }
        Ok(None)
    }

    pub fn print_campaign_result(&self, campaign_result: &CampaignResult) {
        let output = Output::new(&self.contract);
        output.print_campaign_result(campaign_result);
    }
}

pub struct Output<'a> {
    contract: &'a ContractBundle,
}

impl<'a> Output<'a> {
    pub fn new(contract: &'a ContractBundle) -> Self {
        Self { contract }
    }
    pub fn decode_message(&self, data: &Vec<u8>) -> Result<Value> {
        let decoded = self
            .contract
            .transcoder
            .decode_contract_message(&mut data.as_slice())
            .map_err(|e| anyhow::anyhow!("Error decoding message: {:?}", e))?;
        Ok(decoded)
    }

    pub fn decode_deploy(&self, data: &Vec<u8>) -> Result<Value> {
        let decoded = self
            .contract
            .transcoder
            .decode_contract_constructor(&mut data.as_slice())
            .map_err(|e| anyhow::anyhow!("Error decoding constructor: {:?}", e))?;
        Ok(decoded)
    }

    fn print_value(value: &Value) {
        match value {
            Value::Map(map) => {
                print!("{}(", map.ident().unwrap());
                for (n, (_name, value)) in map.iter().enumerate() {
                    if n != 0 {
                        print!(", ");
                    }
                    Self::print_value(value);
                }
                print!(")");
            }
            _ => {
                print!("{:?}", value);
            }
        }
    }

    pub fn print_campaign_result(&self, campaign_result: &CampaignResult) {
        for failed_trace in &campaign_result.failed_traces {
            println!("Property check failed ❌");

            // Contract Deployment
            match self.decode_deploy(&failed_trace.trace.deploy.data) {
                Err(_e) => {
                    println!("Raw deploy: {:?}", &failed_trace.trace.deploy.data);
                }
                Result::Ok(x) => {
                    print!("  Deploy: ",);
                    Self::print_value(&x);
                    println!();
                }
            }

            // Messages
            for (idx, message) in failed_trace.trace.messages.iter().enumerate() {
                print!("  Message{}: ", idx);
                match self.decode_message(&message.input) {
                    Err(_e) => {
                        println!("Raw message: {:?}", &message.input);
                    }
                    Result::Ok(x) => {
                        print!("  Deploy: ",);
                        Self::print_value(&x);
                        println!();
                    }
                }
            }

            // Failed properties
            for message in failed_trace.failed_properties.iter() {
                match self.decode_message(&message.input) {
                    Err(_e) => {
                        println!("Raw message: {:?}", &message.input);
                    }
                    Result::Ok(x) => {
                        print!("  Property: ",);
                        Self::print_value(&x);
                        println!();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test that the hash of two FuzzTraces are equal
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

    // test method info mutates and payable
    #[test]
    fn test_method_info() {
        let arguments = vec![];
        let method_info = MethodInfo::new(arguments, true, true, false);
        assert!(method_info.mutates);
        assert!(method_info.payable);
        assert!(!method_info.constructor);
    }
}
