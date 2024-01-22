use wasmi::core::Trap;

pub trait HostFunctions {
    fn seal0_input(
        &mut self,
        memory: &mut [u8],
        buf_ptr: u32,
        buf_len_ptr: u32,
    ) -> Result<(), Trap>;

    fn seal0_value_transferred(
        &mut self,
        memory: &mut [u8],
        out_ptr: u32,
        out_len_ptr: u32,
    ) -> Result<(), Trap>;

    fn seal0_seal_return(
        &mut self,
        memory: &mut [u8],
        flags: u32,
        data_ptr: u32,
        data_len: u32,
    ) -> Result<(), Trap>;

    fn seal2_set_storage(
        &mut self,
        memory: &[u8],
        key_ptr: u32,
        key_len: u32,
        value_ptr: u32,
        value_len: u32,
    ) -> Result<u32, Trap>;
}

pub trait Seal0HostFunctions {
    fn input(
        &mut self,
        memory: &mut [u8],
        buf_ptr: u32,
        buf_len_ptr: u32,
    ) -> Result<(), Trap>;

    fn value_transferred(
        &mut self,
        memory: &mut [u8],
        out_ptr: u32,
        out_len_ptr: u32,
    ) -> Result<(), Trap>;

    fn seal_return(
        &mut self,
        memory: &mut [u8],
        flags: u32,
        data_ptr: u32,
        data_len: u32,
    ) -> Result<(), Trap>;
}

pub trait Seal1HostFunctions {}

pub trait Seal2HostFunctions {
    fn set_storage(
        &mut self,
        memory: &[u8],
        key_ptr: u32,
        key_len: u32,
        value_ptr: u32,
        value_len: u32,
    ) -> Result<u32, Trap>;
}
