use wasmi::*;
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

#[derive(Debug)]
pub struct HostState {
    pub input_buffer: Vec<u8>,
    pub caller: [u8; 32],
    pub value_transferred: u128,
    pub memory: Option<Memory>,
}
