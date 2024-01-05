//mod environment;
use wasmi::{
    core::ValueType as WasmiValueType, Config as WasmiConfig, Engine, ExternType,
    FuelConsumptionMode, Module, StackLimits,
};
use wasmi::{InstancePre, Linker, Memory, MemoryType, Store};

fn main() {
    println!("Hello, world!");
    type AccountId = [u8; 32];
    type Hash = [u8; 32];
    type Balance = u128;
    type CodeType = Vec<u8>;
    type AllowDeprecatedInterface = bool;
    type AllowUnstableInterface = bool;
    /// Output of a contract call or instantiation which ran to completion.
    //#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    // pub struct ExecReturnValue {
    //     /// Flags passed along by `seal_return`. Empty when `seal_return` was never called.
    //     pub flags: ReturnFlags,
    //     /// Buffer passed along by `seal_return`. Empty when `seal_return` was never called.
    //     pub data: Vec<u8>,
    // }

    // impl ExecReturnValue {
    //     /// The contract did revert all storage changes.
    //     pub fn did_revert(&self) -> bool {
    //         self.flags.contains(ReturnFlags::REVERT)
    //     }
    // }
    // pub type ExecResult = Result<ExecReturnValue, ExecError>;
    /// Can only be used for one call.
    pub struct ContractHostState<'a, E: Ext + 'a> {
        ext: &'a mut E,
        input_data: Option<Vec<u8>>,
        memory: Option<Memory>,
    }

    impl<'a, E: Ext + 'a> ContractHostState<'a, E> {
        pub fn new(ext: &'a mut E, input_data: Vec<u8>) -> Self {
            ContractHostState {
                ext,
                input_data: Some(input_data),
                memory: None,
            }
        }
    }
    /// Imported memory must be located inside this module. The reason for hardcoding is that current
    /// compiler toolchains might not support specifying other modules than "env" for memory imports.
    pub const IMPORT_MODULE_MEMORY: &str = "env";

    const BYTES_PER_PAGE: usize = 64 * 1024;

    const INSTRUCTION_WEIGHTS_BASE: u64 = 1; //TODO: CHECK THIS VALUE

    pub enum Determinism {
        /// The execution should be deterministic and hence no indeterministic instructions are
        /// allowed.
        ///
        /// Dispatchables always use this mode in order to make on-chain execution deterministic.
        Enforced,
        /// Allow calling or uploading an indeterministic code.
        ///
        /// This is only possible when calling into `pallet-contracts` directly via
        /// [`crate::Pallet::bare_call`].
        ///
        /// # Note
        ///
        /// **Never** use this mode for on-chain execution.
        Relaxed,
    }
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
            determinism: Determinism,
            stack_limits: Option<StackLimits>,
        ) -> Result<Self, &'static str> {
            // NOTE: wasmi does not support unstable WebAssembly features. The module is implicitly
            // checked for not having those ones when creating `wasmi::Module` below.
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
                .floats(matches!(determinism, Determinism::Relaxed))
                .consume_fuel(true)
                .fuel_consumption_mode(FuelConsumptionMode::Eager);

            if let Some(stack_limits) = stack_limits {
                config.set_stack_limits(stack_limits);
            }

            let engine = Engine::new(&config);
            let module =
                Module::new(&engine, code).map_err(|_| "Can't load the module into wasmi!")?;

            // Return a `LoadedModule` instance with
            // __valid__ module.
            Ok(LoadedModule { module, engine })
        }
        /// Check that the module has required exported functions. For now
        /// these are just entrypoints:
        ///
        /// - 'call'
        /// - 'deploy'
        ///
        /// Any other exports are not allowed.
        fn scan_exports(&self) -> Result<(), &'static str> {
            let mut deploy_found = false;
            let mut call_found = false;
            let module = &self.module;
            let exports = module.exports();

            for export in exports {
                match export.ty() {
                    ExternType::Func(ft) => {
                        match export.name() {
                            "call" => call_found = true,
                            "deploy" => deploy_found = true,
                            _ => return Err(
                                "unknown function export: expecting only deploy and call functions",
                            ),
                        }
                        // Check the signature.
                        // Both "call" and "deploy" have the () -> () function type.
                        // We still support () -> (i32) for backwards compatibility.
                        if !(ft.params().is_empty()
                            && (ft.results().is_empty() || ft.results() == [WasmiValueType::I32]))
                        {
                            return Err("entry point has wrong signature");
                        }
                    }
                    ExternType::Memory(_) => return Err("memory export is forbidden"),
                    ExternType::Global(_) => return Err("global export is forbidden"),
                    ExternType::Table(_) => return Err("table export is forbidden"),
                }
            }

            if !deploy_found {
                return Err("deploy function isn't exported");
            }
            if !call_found {
                return Err("call function isn't exported");
            }

            Ok(())
        }

        /// Scan an import section if any.
        ///
        /// This makes sure that:
        /// - The import section looks as we expect it from a contract.
        /// - The limits of the memory type declared by the contract comply with the Schedule.
        ///
        /// Returns the checked memory limits back to caller.
        ///
        /// This method fails if:
        ///
        /// - Memory import not found in the module.
        /// - Tables or globals found among imports.
        /// - `call_chain_extension` host function is imported, while chain extensions are disabled.
        ///
        /// NOTE that only single memory instance is allowed for contract modules, which is enforced by
        /// this check combined with multi_memory proposal gets disabled in the engine.
        pub fn scan_imports(&self, mem_pages: u32) -> Result<(u32, u32), &'static str> {
            let module = &self.module;
            let imports = module.imports();
            let mut memory_limits = None;

            for import in imports {
                match *import.ty() {
                    ExternType::Table(_) => return Err("Cannot import tables"),
                    ExternType::Global(_) => return Err("Cannot import globals"),
                    ExternType::Func(_) => {
                        let _ = import.ty().func().ok_or("expected a function")?;
                    }
                    ExternType::Memory(mt) => {
                        if import.module().as_bytes() != IMPORT_MODULE_MEMORY.as_bytes() {
                            return Err("Invalid module for imported memory");
                        }
                        if import.name().as_bytes() != b"memory" {
                            return Err("Memory import must have the field name 'memory'");
                        }
                        if memory_limits.is_some() {
                            return Err("Multiple memory imports defined");
                        }
                        // Parse memory limits defaulting it to (0,0).
                        // Any access to it will then lead to out of bounds trap.
                        let (initial, maximum) = (
                            mt.initial_pages()
                                .to_bytes()
                                .unwrap_or(0)
                                .saturating_div(BYTES_PER_PAGE) as u32,
                            mt.maximum_pages().map_or(mem_pages, |p| {
                                p.to_bytes().unwrap_or(0).saturating_div(BYTES_PER_PAGE) as u32
                            }),
                        );
                        if initial > maximum {
                            return Err(
						"Requested initial number of memory pages should not exceed the requested maximum",
					);
                        }
                        if maximum > mem_pages {
                            return Err("Maximum number of memory pages should not exceed the maximum configured in the Schedule");
                        }

                        memory_limits = Some((initial, maximum));
                        continue;
                    }
                }
            }

            memory_limits.ok_or("No memory import found in the module")
        }
    }

    pub struct CodeInfo {
        /// The account that has uploaded the contract code and hence is allowed to remove it.
        owner: AccountId, //AccountId type
        /// The amount of balance that was deposited by the owner in order to store it on-chain.
        //#[codec(compact)]
        deposit: Balance, //BalanceOf type
        /// The number of instantiated contracts that use this as their code.
        //#[codec(compact)]
        refcount: u64,
        /// Marks if the code might contain non-deterministic features and is therefore never allowed
        /// to be run on-chain. Specifically, such a code can never be instantiated into a contract
        /// and can just be used through a delegate call.
        determinism: Determinism,
        /// length of the code in bytes.
        code_len: u32,
    }

    #[derive(Default)]
    pub struct Diff {
        /// How many bytes were added to storage.
        pub bytes_added: u32,
        /// How many bytes were removed from storage.
        pub bytes_removed: u32,
        /// How many storage items were added to storage.
        pub items_added: u32,
        /// How many storage items were removed from storage.
        pub items_removed: u32,
    }

    pub struct WasmBlob {
        code: CodeType, //type CodeVec<T> = BoundedVec<u8, <T as Config>::MaxCodeLen>;
        // This isn't needed for contract execution and is not stored alongside it.
        //#[codec(skip)]
        code_info: CodeInfo,
        // This is for not calculating the hash every time we need it.
        //#[codec(skip)]
        code_hash: Hash, //<T as frame_system::Config>::Hash;,
    }

    pub fn prepare(
        code: CodeType,
        memory_limit: u32,
        owner: AccountId,
        determinism: Determinism,
    ) -> Result<WasmBlob, ()> {
        //validate::<E, T>(code.as_ref(), schedule, determinism)?; TODO: Check import and exports

        // Calculate deposit for storing contract code and `code_info` in two different storage items.
        let code_len = code.len() as u32;
        //let bytes_added = code_len.saturating_add(<CodeInfo<T>>::max_encoded_len() as u32);
        // let deposit = Diff {
        //     bytes_added,
        //     items_added: 2,
        //     ..Default::default()
        // }
        // .update_contract::<T>(None)
        // .charge_or_zero();
        let code_info = CodeInfo {
            owner,
            deposit: 0,
            determinism,
            refcount: 0,
            code_len,
        };
        //let code_hash = T::Hashing::hash(&code); //TODO: Implement hashing algorithm
        let code_hash = [0; 32];
        Ok(WasmBlob {
            code,
            code_info,
            code_hash,
        })
    }

    pub enum ExportedFunction {
        /// The constructor function which is executed on deployment of a contract.
        Constructor,
        /// The function which is executed when a contract is called.
        Call,
    }

    impl WasmBlob {
        // Create the module by checking the `code`.
        pub fn from_code(
            code: Vec<u8>,
            memory_limit: u32,
            owner: AccountId,
            determinism: Determinism,
        ) -> Result<Self, ()> {
            prepare(code.try_into().map_err(|_| ())?, 10, owner, determinism)
        }

        /// Creates and returns an instance of the supplied code.
        ///
        /// This is either used for later executing a contract or for validation of a contract.
        /// When validating we pass `()` as `host_state`. Please note that such a dummy instance must
        /// **never** be called/executed, since it will panic the executor.
        pub fn instantiate<E, H>(
            code: &[u8],
            host_state: H,
            mem_pages: u32,
            determinism: Determinism,
            stack_limits: StackLimits,
            allow_deprecated: AllowDeprecatedInterface,
        ) -> Result<(Store<H>, Memory, InstancePre), &'static str>
// where
        //     E: Environment<H>,
        {
            let contract = LoadedModule::new(&code, determinism, Some(stack_limits))?;
            let mut store = Store::new(&contract.engine, host_state);
            let mut linker = Linker::new(&contract.engine);
            // E::define(
            //     &mut store,
            //     &mut linker,
            //     if T::UnsafeUnstableInterface::get() {
            //         AllowUnstableInterface::Yes
            //     } else {
            //         AllowUnstableInterface::No
            //     },
            //     allow_deprecated,
            // )
            // .map_err(|_| "can't define host functions to Linker")?;

            // Query wasmi for memory limits specified in the module's import entry.
            let memory_limits = contract.scan_imports(mem_pages)?;
            // Here we allocate this memory in the _store_. It allocates _inital_ value, but allows it
            // to grow up to maximum number of memory pages, if necessary.
            let qed = "We checked the limits versus our Schedule,
					 which specifies the max amount of memory pages
					 well below u16::MAX; qed";
            let memory = Memory::new(
                &mut store,
                MemoryType::new(memory_limits.0, Some(memory_limits.1)).expect(qed),
            )
            .expect(qed);

            linker
                .define("env", "memory", memory)
                .expect("We just created the Linker. It has no definitions with this name; qed");

            let instance = linker
                .instantiate(&mut store, &contract.module)
                .map_err(|_| "can't instantiate module with provided definitions")?;

            Ok((store, memory, instance))
        }

        //fn execute<E: Ext<T = T>>(
        fn execute(
            self,
            //ext: &mut E,
            function: &ExportedFunction,
            input_data: Vec<u8>,
        ) -> ExecResult {
            let code = self.code.as_slice();
            // Instantiate the Wasm module to the engine.
            let runtime = Runtime::new(ext, input_data);
            let schedule = <T>::Schedule::get();
            let (mut store, memory, instance) = Self::instantiate(
                code,
                runtime,
                &schedule,
                self.code_info.determinism,
                StackLimits::default(),
                match function {
                    ExportedFunction::Call => true,
                    ExportedFunction::Constructor => false,
                },
            )
            .map_err(|msg| {
                //log::debug!(target: LOG_TARGET, "failed to instantiate code to wasmi: {}", msg);
                ()
            })?;
            store.data_mut().set_memory(memory);

            // Set fuel limit for the wasmi execution.
            // We normalize it by the base instruction weight, as its cost in wasmi engine is `1`.
            let fuel_limit = store
                .data_mut()
                .ext()
                .gas_meter_mut()
                .gas_left()
                .ref_time()
                .checked_div(INSTRUCTION_WEIGHTS_BASE)
                .ok_or(())?;
            store
                .add_fuel(fuel_limit)
                .expect("We've set up engine to fuel consuming mode; qed");

            // Sync this frame's gas meter with the engine's one.
            let process_result = |mut store: Store<Runtime<E>>, result| {
                let engine_consumed_total = store
                    .fuel_consumed()
                    .expect("Fuel metering is enabled; qed");
                let gas_meter = store.data_mut().ext().gas_meter_mut();
                gas_meter.charge_fuel(engine_consumed_total)?;
                store.into_data().to_execution_result(result)
            };

            // Start function should already see the correct refcount in case it will be ever inspected.
            if let &ExportedFunction::Constructor = function {
                E::increment_refcount(self.code_hash)?;
            }

            // Any abort in start function (includes `return` + `terminate`) will make us skip the
            // call into the subsequent exported function. This means that calling `return` returns data
            // from the whole contract execution.
            match instance.start(&mut store) {
                Ok(instance) => {
                    let exported_func = instance
                        .get_export(&store, function.identifier())
                        .and_then(|export| export.into_func())
                        .ok_or_else(|| {
                            //log::error!(target: LOG_TARGET, "failed to find entry point");
                            //Error::<T>::CodeRejected
                            ()
                        })?;

                    let result = exported_func.call(&mut store, &[], &mut []);
                    process_result(store, result)
                }
                Err(err) => process_result(store, Err(err)),
            }
        }
    }
}

#[cfg(test)]
pub mod wasm_test {
    // use pallet_contracts::wasm::WasmBlob;
    // use pallet_contracts::*;
    // #[test]
    // fn testing() {
    //     let wat = r#"
    //     (module
    //         (import "host" "hello" (func $host_hello (param i32)))
    //         (func (export "hello")
    //             (call $host_hello (i32.const 3))
    //         )
    //     )
    // "#;
    //     // Wasmi does not yet support parsing `.wat` so we have to convert
    //     // out `.wat` into `.wasm` before we compile and validate it.
    //     let wasm = wat::parse_str(wat).unwrap();
    //     let wasm_blob =
    //         WasmBlob::<Test>::from_code_unchecked(wasm, Default::default(), Default::default());
    // }
}
