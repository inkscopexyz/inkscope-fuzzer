use parity_scale_codec::{Decode, DecodeLimit, Encode, MaxEncodedLen};
use std::collections::HashMap;
use wasmi::core::Trap;
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
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    pub input_buffer: Vec<u8>,
    pub caller: [u8; 32],
    pub value_transferred: u128,
    pub memory: Option<Memory>,
    pub return_data: Option<Vec<u8>>,
}
/// The maximum nesting depth a contract can use when encoding types.
const MAX_DECODE_NESTING: u32 = 256;

impl HostState {
    pub fn get_input(&self) -> &[u8] {
        &self.input_buffer
    }

    /// Reads and decodes a type with a size fixed at compile time from contract memory.
    pub fn decode_from_memory<D: Decode + MaxEncodedLen + std::fmt::Debug>(
        &self,
        memory: &mut [u8],
        ptr: u32,
    ) -> Result<D, Trap> {
        let ptr = ptr as usize;
        let mut bound_checked =
            memory
                .get(ptr..ptr + D::max_encoded_len() as usize)
                .ok_or(Trap::new(format!(
                    "Pointer out of bound reading at {}",
                    ptr
                )))?;

        D::decode_with_depth_limit(MAX_DECODE_NESTING, &mut bound_checked)
            .map_err(|_| Trap::new(format!("Error decoding at {}", ptr)))
    }

    pub fn encode_to_memory<E: Encode + MaxEncodedLen + std::fmt::Debug>(
        &self,
        memory: &mut [u8],
        ptr: u32,
        value: E,
    ) -> Result<(), Trap> {
        let buffer = value.encode();
        self.write_to_memory(memory, ptr, &buffer)
    }

    pub fn encode_to_memory_bounded<E: Encode + MaxEncodedLen + std::fmt::Debug>(
        &self,
        memory: &mut [u8],
        ptr: u32,
        ptr_len: u32,
        value: E,
    ) -> Result<(), Trap> {
        let len = self.decode_from_memory::<u32>(memory, ptr_len)?;
        let buffer = value.encode();
        let buffer_len = buffer.len();
        if buffer_len > len as usize {
            return Err(Trap::new(format!(
                "Buffer too large to encode at {} with size {}",
                ptr, len
            )));
        }
        self.write_to_memory(memory, ptr, &buffer)?;
        self.write_to_memory(memory, ptr_len, &(buffer_len as u32).encode())
    }

    /// Write the given buffer to the designated location in the memory.
    pub fn write_to_memory(&self, memory: &mut [u8], ptr: u32, buf: &[u8]) -> Result<(), Trap> {
        let ptr = ptr as usize;
        let bound_checked = memory
            .get_mut(ptr..ptr + buf.len())
            .ok_or(Trap::new(format!(
                "Pointer out of bound writing at {}",
                ptr
            )))?;

        bound_checked.copy_from_slice(buf);
        Ok(())
    }

    /// read from the designated location in the memory.
    pub fn read_from_memory<'a>(
        &self,
        memory: &'a [u8],
        ptr: u32,
        len: u32,
    ) -> Result<&'a [u8], Trap> {
        let ptr = ptr as usize;

        let mut bound_checked = memory
            .get(ptr..ptr + len as usize)
            .ok_or(Trap::new(format!(
                "Pointer out of bound reading at {}",
                ptr
            )))?;

        Ok(bound_checked)
    }

    pub fn set_storage(
        &mut self,
        memory: &[u8],
        key_ptr: u32,
        key_len: u32,
        value_ptr: u32,
        value_len: u32,
    ) -> Result<u32, Trap> {
        let key = self.read_from_memory(memory, key_ptr, key_len)?;
        let value = self.read_from_memory(memory, value_ptr, value_len)?;
        self.storage.insert(key.into(), value.into());

        Ok(0)
    }

    pub fn set_return_data(&mut self, return_data: &[u8]) {
        self.return_data = Some(return_data.into());
    }

    /// Stores the input passed by the caller into the supplied buffer.
    ///
    /// The value is stored to linear memory at the address pointed to by `out_ptr`.
    /// `out_len_ptr` must point to a u32 value that describes the available space at
    /// `out_ptr`. This call overwrites it with the size of the value. If the available
    /// space at `out_ptr` is less than the size of the value a trap is triggered.
    ///
    pub fn seal0_input(
        &mut self,
        memory: &mut [u8],
        buf_ptr: u32,
        buf_len_ptr: u32,
    ) -> Result<(), Trap> {
        self.decode_from_memory::<u32>(memory, buf_len_ptr)?;

        // TODO generate approiate inpud using host state and seed and abi and whatever
        let input = self.get_input();
        let input_len = u32::try_from(input.len()).expect("Buffer length must be less than 4Gigs");

        self.write_to_memory(memory, buf_ptr, input)?;
        self.encode_to_memory(memory, buf_len_ptr, input_len)
    }
}



