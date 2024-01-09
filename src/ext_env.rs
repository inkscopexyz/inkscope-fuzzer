use wasmi::*;
use parity_scale_codec::{Decode, MaxEncodedLen, DecodeLimit};
use wasmi::core::Trap;


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
/// The maximum nesting depth a contract can use when encoding types.
const MAX_DECODE_NESTING: u32 = 256;

impl HostState{

    /// Reads and decodes a type with a size fixed at compile time from contract memory.
    pub fn decode_from_memory_as<D: Decode + MaxEncodedLen + std::fmt::Debug>(
        &self,
        memory: &mut [u8],
        ptr: u32,
    ) -> Result<D, Error> {
        let ptr = ptr as usize;
        let mut bound_checked = memory
            .get(ptr..ptr + D::max_encoded_len() as usize)
            .ok_or_else(|| wasmi::Error::Memory(errors::MemoryError::OutOfBoundsAccess));

        println!("bound_checked: {:?}", bound_checked);

        let mut bound_checked =     bound_checked ?;

        let decoded = D::decode_with_depth_limit(MAX_DECODE_NESTING, &mut bound_checked)
            .map_err(|_| wasmi::Error::Trap(Trap::new(format!("Error decoding at {}", ptr))));
        println!("decoded: {:?}", decoded);
        let decoded = decoded?;
        Ok(decoded)
    }

    // pub fn write_to_memory(&self, buffer: &[u8], ptr: u32) -> Result<(), wasmi::Error>{
    //     let memory = self.memory.ok_or(Trap::new("No memory"))?.data_mut(&mut ctx);
    // }
 

}