use parity_scale_codec::{
    Decode,
    DecodeLimit,
    Encode,
    MaxEncodedLen,
};
use std::collections::HashMap;
use wasmi::{
    core::Trap,
    *,
};

pub struct LoadedModule {
    pub module: Module,
    pub engine: Engine,
}

impl LoadedModule {
    /// Creates a new instance of `LoadedModule`.
    pub fn new(
        code: &[u8],
        determinism: bool,
        stack_limits: Option<StackLimits>,
    ) -> Result<Self, &'static str> {
        // NOTE: wasmi does not support unstable WebAssembly features. The module is
        // implicitly checked for not having those ones when creating
        // `wasmi::Module` below.
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
        let module = Module::new(&engine, code)
            .map_err(|_| "Can't load the module into wasmi!")?;

        // Return a `LoadedModule` instance with
        // __valid__ module.
        Ok(LoadedModule { module, engine })
    }
}

pub trait HostFunctions {
    fn seal0_input(
        &mut self,
        memory: &mut [u8],
        buf_ptr: u32,
        buf_len_ptr: u32,
    ) -> Result<(), Trap>;

    fn seal2_set_storage(
        &mut self,
        memory: &[u8],
        key_ptr: u32,
        key_len: u32,
        value_ptr: u32,
        value_len: u32,
    ) -> Result<u32, Trap>;

    fn seal0_value_transferred(
        &mut self,
        memory: &mut [u8],
        out_ptr: u32,
        out_len_ptr: u32,
    ) -> Result<(), Trap>;
}

#[derive(Debug, Clone)]
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

    #[allow(dead_code)]
    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input_buffer = input;
    }

    /// Reads and decodes a type with a size fixed at compile time from contract memory.
    pub fn decode_from_memory<D: Decode + MaxEncodedLen + std::fmt::Debug>(
        &self,
        memory: &mut [u8],
        ptr: u32,
    ) -> Result<D, Trap> {
        let ptr = ptr as usize;
        let mut bound_checked = memory
            .get(ptr..ptr + D::max_encoded_len())
            .ok_or(Trap::new(format!("Pointer out of bound reading at {}", ptr)))?;

        D::decode_with_depth_limit(MAX_DECODE_NESTING, &mut bound_checked)
            .map_err(|_| Trap::new(format!("Error decoding at {}", ptr)))
    }

    pub fn encode_to_memory<E: Encode + MaxEncodedLen + std::fmt::Debug>(
        &self,
        memory: &mut [u8],
        ptr: u32,
        value: E,
    ) -> Result<usize, Trap> {
        let buffer = value.encode();
        self.write_to_memory(memory, ptr, &buffer)?;
        Ok(buffer.len())
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
    pub fn write_to_memory(
        &self,
        memory: &mut [u8],
        ptr: u32,
        buf: &[u8],
    ) -> Result<(), Trap> {
        let ptr = ptr as usize;
        let bound_checked = memory
            .get_mut(ptr..ptr + buf.len())
            .ok_or(Trap::new(format!("Pointer out of bound writing at {}", ptr)))?;

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

        let bound_checked = memory
            .get(ptr..ptr + len as usize)
            .ok_or(Trap::new(format!("Pointer out of bound reading at {}", ptr)))?;

        Ok(bound_checked)
    }

    pub fn set_return_data(&mut self, return_data: &[u8]) {
        self.return_data = Some(return_data.into());
    }
}

impl HostFunctions for HostState {
    /// Stores the input passed by the caller into the supplied buffer.
    fn seal0_input(
        &mut self,
        memory: &mut [u8],
        buf_ptr: u32,
        buf_len_ptr: u32,
    ) -> Result<(), Trap> {
        let requested_bytes = self.decode_from_memory::<u32>(memory, buf_len_ptr)?;

        // TODO generate approiate inpud using host state and seed and abi and whatever
        let input = self.get_input();
        let input_len =
            u32::try_from(input.len()).expect("Buffer length must be less than 4Gigs");

        if input_len > requested_bytes {
            return Err(Trap::new(format!(
                "Requested {} bytes, but only {} bytes available",
                requested_bytes, input_len
            )));
        }

        self.write_to_memory(memory, buf_ptr, input)?;
        self.encode_to_memory(memory, buf_len_ptr, input_len)?;
        Ok(())
    }

    /// Set the value at the given key in the contract storage.
    fn seal2_set_storage(
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

    /// Stores the value transferred along with this call/instantiate into the supplied
    /// buffer.
    ///
    /// The value is stored to linear memory at the address pointed to by `out_ptr`.
    /// `out_len_ptr` must point to a `u32` value that describes the available space at
    /// `out_ptr`. This call overwrites it with the size of the value. If the available
    /// space at `out_ptr` is less than the size of the value a trap is triggered.
    ///
    /// The data is encoded as `T::Balance`.
    fn seal0_value_transferred(
        &mut self,
        memory: &mut [u8],
        out_ptr: u32,
        out_len_ptr: u32,
    ) -> Result<(), Trap> {
        self.encode_to_memory_bounded(
            memory,
            out_ptr,
            out_len_ptr,
            self.value_transferred,
        )
    }
}
#[cfg(test)]
mod test {
    use super::*;
    pub mod seal0_input {
        use super::*;
        #[test]
        fn happy_path() {
            let buf_len_ptr = 0;
            let read_size = 1000;

            let mut host_state = HostState {
                storage: HashMap::new(),
                input_buffer: vec![1, 2, 3, 4],
                caller: [0; 32],
                value_transferred: 0,
                memory: None,
                return_data: None,
            };

            // Dumb memory
            let mut mem = [0; 1024];
            println!("mem: {:?}", mem);

            let buf_len_ptr_size = host_state
                .encode_to_memory(mem.as_mut_slice(), buf_len_ptr, read_size)
                .expect("This should work");

            println!("mem: {:?}", mem);
            assert_eq!(buf_len_ptr_size, read_size.encode().len()); // At the time of writing this is 4 bytes, same as u32::max_encoded_len()

            host_state
                .seal0_input(mem.as_mut_slice(), buf_len_ptr_size as u32, buf_len_ptr)
                .expect("Pls dont fail");

            println!("mem: {:?}", mem);
            let buf_ptr_size: u32 = host_state
                .decode_from_memory::<u32>(mem.as_mut_slice(), buf_len_ptr)
                .expect("Pls dont fail");

            assert_eq!(buf_ptr_size, host_state.input_buffer.len() as u32);

            assert!(
                mem[buf_len_ptr_size..buf_len_ptr_size + host_state.input_buffer.len()]
                    == host_state.input_buffer
            )
        }

        #[test]
        fn buf_smaller_than_input() {
            let buf_len_ptr = 0;
            let read_size = 2; // This is smaller than the input buffer len

            let mut host_state = HostState {
                storage: HashMap::new(),
                input_buffer: vec![1, 2, 3, 4],
                caller: [0; 32],
                value_transferred: 0,
                memory: None,
                return_data: None,
            };

            // Dumb memory
            let mut mem = [0; 1024];
            println!("mem: {:?}", mem);

            let buf_len_ptr_size = host_state
                .encode_to_memory(mem.as_mut_slice(), buf_len_ptr, read_size)
                .expect("This should work");

            println!("mem: {:?}", mem);
            assert_eq!(buf_len_ptr_size, read_size.encode().len());

            assert!(host_state
                .seal0_input(mem.as_mut_slice(), buf_len_ptr_size as u32, buf_len_ptr)
                .is_err());
        }

        #[test]
        fn buf_out_of_bound() {
            let buf_len_ptr = 0;
            let read_size = 6; // This is smaller than the input buffer len

            let mut host_state = HostState {
                storage: HashMap::new(),
                input_buffer: vec![1, 2, 3, 4],
                caller: [0; 32],
                value_transferred: 0,
                memory: None,
                return_data: None,
            };

            // Dumb memory
            let mut mem = [0; 4];
            println!("mem: {:?}", mem);

            let buf_len_ptr_size = host_state
                .encode_to_memory(mem.as_mut_slice(), buf_len_ptr, read_size)
                .expect("This should work");

            println!("mem: {:?}", mem);
            assert_eq!(buf_len_ptr_size, read_size.encode().len());

            let result = host_state.seal0_input(
                mem.as_mut_slice(),
                buf_len_ptr_size as u32,
                buf_len_ptr,
            );

            match result {
                Ok(_) => panic!("Should have failed"),
                Err(e) => {
                    println!("e: {:?}", e);
                    assert!(e.to_string().contains("Pointer out of bound writing at"))
                }
            }
        }
    }
}
