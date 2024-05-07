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
    bail,
    Ok,
    Result,
};
use contract_transcode::Value;
use ink_sandbox::{
    api::{
        balance_api::BalanceAPI,
        contracts_api::ContractAPI,
    },
    frame_support::{
        print,
        sp_runtime::traits::Hash,
    },
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
    path::PathBuf, sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub enum CampaignStatus{
    Initializing,
    InProgress,
    Finished,
}

#[derive(Debug, Clone)]
pub struct CampaignData {
    pub properties: Vec<String>,
    pub seed: u64,
    pub failed_traces: Vec<FailedTrace>,
    pub status: CampaignStatus
}
impl Default for CampaignData {
    fn default() -> Self {
       Self{
              properties: vec![],
              seed: 0,
              failed_traces: vec![],
             status: CampaignStatus::Initializing
       }
    }
}

pub struct CampaignResult {
    pub failed_traces: Vec<FailedTrace>,
}

#[derive(Debug, Clone)]
pub struct FailedTrace {
    /// The trace that failed
    pub trace: Trace,
    /// The failing reason (see FailReason)
    pub reason: FailReason,
}
impl FailedTrace {
    pub fn new(trace: Trace, reason: FailReason) -> Self {
        Self { trace, reason }
    }
}

// Our own copy of method information. The selector is used as the key in the hashmap
struct MethodInfo {
    method_name: String,
    arguments: Vec<TypeDef<PortableForm>>,
    #[allow(dead_code)]
    mutates: bool,
    payable: bool,
    #[allow(dead_code)]
    constructor: bool,
}

impl MethodInfo {
    fn new(
        method_name: String,
        arguments: Vec<TypeDef<PortableForm>>,
        mutates: bool,
        payable: bool,
        constructor: bool,
    ) -> Self {
        Self {
            method_name,
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

#[derive(Debug, Clone, Hash)]
enum DeployOrMessage {
    Deploy(Deploy),
    Message(Message),
}
impl DeployOrMessage {
    pub fn data(&self) -> &Vec<u8> {
        match self {
            DeployOrMessage::Deploy(deploy) => &deploy.data,
            DeployOrMessage::Message(message) => &message.input,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Trace {
    messages: Vec<DeployOrMessage>,
}

impl Trace {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    // This function should be used to push a new Message to the trace
    fn push(&mut self, deploy_or_message: DeployOrMessage) {
        self.messages.push(deploy_or_message);
    }

    fn hash(&self) -> TraceHash {
        let mut hasher = DefaultHasher::new();
        self.messages.hash(&mut hasher);
        hasher.finish()
    }

    pub fn contract(&self) -> Result<&AccountId> {
        match self.messages.first() {
            Some(deploy) => {
                match deploy {
                    DeployOrMessage::Deploy(deploy) => Ok(&deploy.address),
                    DeployOrMessage::Message(_) => {
                        Err(anyhow!("First message in the trace is not a deployment"))
                    }
                }
            }
            _ => Err(anyhow!("First message in the trace is not a deployment")),
        }
    }

    fn last_message(&self) -> Option<&DeployOrMessage> {
        self.messages.last()
    }
}

#[derive(Debug)]
pub enum DeployOrMessageResult {
    Trapped,
    Reverted,
    Success(Vec<u8>),
    Unhandled(DispatchError),
}

#[derive(Debug)]
pub enum TraceResult {
    Pass(Option<Vec<u8>>),
    Failed(FailReason),
    Reverted,
}

#[derive(Debug, Clone)]
pub enum FailReason {
    Trapped,
    Property(Message),
}

impl From<Result<ExecReturnValue, DispatchError>> for DeployOrMessageResult {
    fn from(val: Result<ExecReturnValue, DispatchError>) -> Self {
        match val {
            Err(e) => {
                // If the deployment panics, we consider it failed
                match e {
                    DispatchError::Module(module_error)
                        if (module_error.message == Some("ContractTrapped")) =>
                    {
                        DeployOrMessageResult::Trapped
                    }
                    _ => DeployOrMessageResult::Unhandled(e),
                }
            }
            // Return if execution reverted in constructor
            Result::Ok(res) => {
                if !res.flags.is_empty() {
                    DeployOrMessageResult::Reverted
                } else {
                    DeployOrMessageResult::Success(res.data)
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
    snapshot_cache: SnapshotCache,

    // Settings
    config: Config,
}

type SnapshotCache = HashMap<TraceHash, TraceState>;

// #[derive(Debug)]
// pub enum MessageOrDeployResult {
//     Trapped,
//     Reverted,
//     Success(Vec<u8>),
//     Unhandled(DispatchError),
// }

#[derive(Debug)]
struct TraceState {
    /// A Sandbox snapshot containing the state at current trace
    snapshot: Snapshot,

    // The trace result
    result: TraceResult,
}
impl TraceState {
    pub fn new(snapshot: Snapshot, result: TraceResult) -> Self {
        Self { snapshot, result }
    }
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
            let method_info = MethodInfo::new(spec.label().to_string(), arguments, true, spec.payable, true);
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
                MethodInfo::new(spec.label().to_string(), arguments, spec.mutates(), spec.payable(), false);
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
    ) -> DeployOrMessageResult {
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
    ) -> DeployOrMessageResult {
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

    fn execute_deploy_or_message(
        &self,
        sandbox: &mut DefaultSandbox,
        deploy_or_message: &DeployOrMessage,
    ) -> DeployOrMessageResult {
        match deploy_or_message {
            DeployOrMessage::Deploy(deploy) => self.execute_deploy(sandbox, deploy),
            DeployOrMessage::Message(message) => self.execute_message(sandbox, message),
        }
    }

    // Error if a property fail
    fn check_properties(
        &self,
        fuzzer: &mut Fuzzer,
        sandbox: &mut DefaultSandbox,
        trace: &Trace,
    ) -> Result<Vec<Message>> {
        let mut failed_properties = vec![];
        let contract_address = trace.contract()?;

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
                    DeployOrMessageResult::Unhandled(e) => {
                        // If the error is not a ContractTrapped, we panic because
                        // is not an expected behavior
                        panic!("Error: {:?}", e);
                    }
                    DeployOrMessageResult::Trapped | DeployOrMessageResult::Reverted => {
                        // If the property reverts or panics, we also consider it failed
                        failed_properties.push(property_message);
                        break;
                    }
                    DeployOrMessageResult::Success(data) => {
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

    pub fn optimize(&self, fuzzer: &mut Fuzzer, result: FailedTrace) -> FailedTrace {
        let new_trace = Trace::new();
        let remove_idx = fuzzer.rng.usize(1..result.trace.messages.len());
        let mut sandbox = DefaultSandbox::default();

        for (pos, deploy_or_message) in result.trace.messages.iter().enumerate() {
            if pos != remove_idx {
                // new_trace.push(deploy_or_message.clone());
                // let result =
                // self.execute_deploy_or_message_and_check_properties(sandbox,
                // deploy_or_message); match result {

                // }
                todo!()
            }
        }
        todo!()
    }

    pub fn run_campaign(&mut self, campaign_data: &mut Arc<RwLock<CampaignData>>) -> Result<CampaignResult> {
        // Set the seed and properties in the campaign data
        campaign_data.write().unwrap().seed = self.config.seed;
        campaign_data.write().unwrap().properties = self
            .method_info
            .iter()
            .filter(|(selector, _)| self.properties.contains(selector.clone()))
            .map(|(_selector, method_info)| method_info.method_name.clone())
            .collect();
        
        let max_iterations = self.config.max_rounds;
        let fail_fast = self.config.fail_fast;
        let rng_seed = self.config.seed;

        let start_time = std::time::Instant::now();

        let mut failed_traces: Vec<FailedTrace> = vec![];
        let mut fuzzer = Fuzzer::new(rng_seed, self.config.constants.clone());

        for _ in 0..max_iterations {
            let mut local_fuzzer = fuzzer.fork();
            let mut local_snapshot_cache = SnapshotCache::new();
            let new_failed_traces =
                self.run(&mut local_fuzzer, &mut local_snapshot_cache)?;

            failed_traces.extend(new_failed_traces);

            // TODO: Only update the campaign data if new failed traces are found.
            // TODO: Maybe send a message to worker thread to make checks and perform updates
            campaign_data.write().unwrap().failed_traces = failed_traces.clone();

            // If we have failed traces and fail_fast is enabled, we stop the campaign
            if !failed_traces.is_empty() && fail_fast {
                break;
            }
            self.snapshot_cache.extend(local_snapshot_cache);
        }

        println!("Elapsed time: {:?}", start_time.elapsed());
        Ok(CampaignResult { failed_traces })
    }

    fn init<'a>(
        &'a self,
        sandbox: &mut DefaultSandbox,
        local_snapshot_cache: &mut SnapshotCache,
    ) -> Result<Option<&'a Snapshot>> {
        /// Hardcoded empty trace hash
        const EMPTY_TRACE_HASH: u64 = 0;
        // Check if the initial state is already in the cache
        match self.snapshot_cache.get(&EMPTY_TRACE_HASH) {
            Some(cache_entry) => {
                match &cache_entry.result {
                    TraceResult::Pass(_) => Ok(Some(&cache_entry.snapshot)),
                    TraceResult::Failed(reason) => panic!("This should not happen. Failed Initialization should never be saved in the cache {:?}", reason),
                    TraceResult::Reverted => panic!("This should not happen. Reverted Initialization should never be saved in the cache"),
                }
            }
            _ => {
                self.initialize_state(sandbox)?;
                local_snapshot_cache.insert(
                    EMPTY_TRACE_HASH,
                    TraceState::new(sandbox.take_snapshot(), TraceResult::Pass(None)),
                );
                Ok(None)
            }
        }
    }

    fn execute_last<'a>(
        &'a self,
        fuzzer: &mut Fuzzer,
        sandbox: &mut DefaultSandbox,
        local_snapshot_cache: &mut SnapshotCache,
        current_snapshot: &mut Option<&'a Snapshot>,
        trace: &Trace,
    ) -> Result<Vec<FailedTrace>> {
        let mut failed_traces = vec![];

        // CACHE: Check we happened to choose the same constructor as a previous run
        match self.snapshot_cache.get(&trace.hash()) {
            Some(cache_entry) => {
                match &cache_entry.result {
                    TraceResult::Pass(_output) => {
                        debug!("Cahe HIT: Same constructor was choosen and executed before, reloading state from cache");
                        // The trace was already in the cache set current pending state
                        *current_snapshot = Some(&cache_entry.snapshot);
                    }
                    TraceResult::Failed(reason) => {
                        failed_traces
                            .push(FailedTrace::new(trace.clone(), reason.clone()))
                    }
                    TraceResult::Reverted => {
                        // If the message reverts we just ignore this execution and
                        // continue
                    }
                }
            }
            None => {
                debug!("Cahe MISS: The choosen constructor was never executed before. Executing it.");
                // The trace was not in the cache, apply the previous state if any
                if let Some(snapshot) = current_snapshot {
                    debug!("The current state is not yet materialized in the sandbox, restoring current state.");
                    sandbox.restore_snapshot(snapshot.clone());
                }; // Note: is current_snapshot is none then the sandbox must be up to date.

                *current_snapshot = None;

                // Execute the action
                let message_or_deploy_result = match trace.last_message() {
                    Some(last_message) => {
                        self.execute_deploy_or_message(sandbox, last_message)
                    }
                    None => bail!("Empty trace!"),
                };

                match message_or_deploy_result {
                    DeployOrMessageResult::Unhandled(e) => {
                        // If the error is not a ContractTrapped, we panic because
                        // is not an expected behavior
                        panic!("Error: {:?}", e);
                    }
                    DeployOrMessageResult::Trapped => {
                        local_snapshot_cache.insert(
                            trace.hash(),
                            TraceState::new(
                                sandbox.take_snapshot(),
                                TraceResult::Failed(FailReason::Trapped),
                            ),
                        );
                        failed_traces.push(FailedTrace {
                            trace: trace.clone(),
                            reason: FailReason::Trapped,
                        });
                    }
                    DeployOrMessageResult::Reverted => {
                        // Note: the snapshot here is probably never used as the last tx
                        // reverted
                        local_snapshot_cache.insert(
                            trace.hash(),
                            TraceState::new(
                                sandbox.take_snapshot(),
                                TraceResult::Reverted,
                            ),
                        );
                        // If the message reverts we just ignore this execution and
                    }
                    DeployOrMessageResult::Success(output) => {
                        // If it did not revert or panic we check the properties
                        let failed_properties =
                            self.check_properties(fuzzer, sandbox, &trace)?;
                        if !failed_properties.is_empty() {
                            for failed_property in &failed_properties {
                                debug!("Property check failed: {:?}", failed_property);
                                local_snapshot_cache.insert(
                                    trace.hash(),
                                    TraceState::new(
                                        sandbox.take_snapshot(),
                                        TraceResult::Failed(FailReason::Property(
                                            failed_property.clone(),
                                        )),
                                    ),
                                );

                                failed_traces.push(FailedTrace {
                                    trace: trace.clone(),
                                    reason: FailReason::Property(
                                        failed_property.to_owned(),
                                    ),
                                });
                            }
                        } else {
                            // If the execution went ok then store the new state in the
                            // cache
                            local_snapshot_cache.insert(
                                trace.hash(),
                                TraceState::new(
                                    sandbox.take_snapshot(),
                                    TraceResult::Pass(Some(output)),
                                ),
                            );
                        }
                    }
                }
            }
        };
        Ok(failed_traces)
    }

    fn run(
        &self,
        fuzzer: &mut Fuzzer,
        local_snapshot_cache: &mut SnapshotCache,
    ) -> Result<Vec<FailedTrace>> {
        debug!("Starting run");

        // Local mutable state...
        // Sandbox for the emulation
        let mut sandbox = DefaultSandbox::default();
        let mut current_snapshot = self.init(&mut sandbox, local_snapshot_cache)?;

        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // Deploy the main contract to be fuzzed using a random constructor with fuzzed
        // argumets
        let constructor_selector = fuzzer.choice(&self.constructors).unwrap();
        let constructor =
            self.generate_constructor(fuzzer, constructor_selector, Default::default())?;

        // Start the trace with a deployment
        let mut trace = Trace::new();
        trace.push(DeployOrMessage::Deploy(constructor));

        let mut failed_traces = self.execute_last(
            fuzzer,
            &mut sandbox,
            local_snapshot_cache,
            &mut current_snapshot,
            &trace,
        )?;

        // If the deployment failed, we return the failed trace. We do not continue
        if !failed_traces.is_empty() {
            return Ok(failed_traces);
        }

        let max_txs = self.config.max_number_of_transactions;
        let callee = trace.contract()?.clone();
        for i in 0..max_txs {
            debug!("Tx: {}/{}", i, max_txs);

            let message_selector = fuzzer.choice(&self.messages).unwrap();
            let message = self.generate_message(fuzzer, message_selector, &callee)?;

            trace.push(DeployOrMessage::Message(message));

            failed_traces.extend(self.execute_last(
                fuzzer,
                &mut sandbox,
                local_snapshot_cache,
                &mut current_snapshot,
                &trace,
            )?);
        }
        Ok(failed_traces)
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

            // Messages
            for (idx, deploy_or_message) in failed_trace.trace.messages.iter().enumerate()
            {
                print!("  Message{}: ", idx);
                let decode_result = match deploy_or_message {
                    DeployOrMessage::Deploy(deploy) => self.decode_deploy(&deploy.data),
                    DeployOrMessage::Message(message) => {
                        self.decode_message(&message.input)
                    }
                };
                match decode_result {
                    Err(_e) => {
                        println!("Raw message: {:?}", &deploy_or_message.data());
                    }
                    Result::Ok(x) => {
                        print!("  Message: ",);
                        Self::print_value(&x);
                        println!();
                    }
                }
            }

            match &failed_trace.reason {
                FailReason::Trapped => println!("Last message in trace has Trapped"),
                FailReason::Property(failed_property) => {
                    // Failed properties

                    match self.decode_message(&failed_property.input) {
                        Err(_e) => {
                            println!("Raw message: {:?}", &failed_property.input);
                        }
                        Result::Ok(x) => {
                            print!("  Property: ",);
                            Self::print_value(&x);
                            println!();
                        }
                    }
                }
            };
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
        let mut trace1 = Trace::new();
        let mut trace2 = Trace::new();
        assert_eq!(&trace1.hash(), &trace2.hash());

        trace1.push(DeployOrMessage::Deploy(deploy.clone()));
        trace2.push(DeployOrMessage::Deploy(deploy));
        assert_eq!(&trace1.hash(), &trace2.hash());

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

        trace1.push(DeployOrMessage::Message(message));
        trace2.push(DeployOrMessage::Message(message_identical));
        assert_eq!(&trace1.hash(), &trace2.hash());
    }

    // test method info mutates and payable
    #[test]
    fn test_method_info() {
        let arguments = vec![];
        let method_info = MethodInfo::new(String::from("Name"), arguments, true, true, false);
        assert!(method_info.mutates);
        assert!(method_info.payable);
        assert!(!method_info.constructor);
    }
}
