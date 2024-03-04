use drink::{
    frame_system,
    runtime::{AccountIdFor, HashFor},
    BalanceOf, MinimalRuntime,
};
use ink_metadata::{ConstructorSpec, MessageSpec};
use scale_info::form::PortableForm;

//TODO: add this to drink/runtime.rs
pub type HashingFor<R> = <R as frame_system::Config>::Hashing;

//This defines all the configurable types based on the current runtime: MinimalRuntime
pub type Balance = BalanceOf<MinimalRuntime>;
pub type AccountId = AccountIdFor<MinimalRuntime>;
pub type Hash = HashFor<MinimalRuntime>;
pub type CodeHash = HashFor<MinimalRuntime>;
pub type Hashing = HashingFor<MinimalRuntime>;
pub type TraceHash = u64;

pub enum ConstructorOrMessageSpec<'a> {
    Constructor(&'a ConstructorSpec<PortableForm>),
    Message(&'a MessageSpec<PortableForm>),
}

impl ConstructorOrMessageSpec<'_> {
    pub fn label(&self) -> &str {
        match self {
            ConstructorOrMessageSpec::Constructor(spec) => spec.label(),
            ConstructorOrMessageSpec::Message(spec) => spec.label(),
        }
    }

    pub fn selector(&self) -> [u8; 4] {
        let selector = match self {
            ConstructorOrMessageSpec::Constructor(spec) => spec.selector(),
            ConstructorOrMessageSpec::Message(spec) => spec.selector(),
        };
        let mut result = [0u8; 4];
        result.copy_from_slice(selector.to_bytes());
        result
    }

    pub fn args(&self) -> &[ink_metadata::MessageParamSpec<PortableForm>] {
        match self {
            ConstructorOrMessageSpec::Constructor(spec) => spec.args(),
            ConstructorOrMessageSpec::Message(spec) => spec.args(),
        }
    }

    pub fn payable(&self) -> bool {
        match self {
            ConstructorOrMessageSpec::Constructor(spec) => *spec.payable(),
            ConstructorOrMessageSpec::Message(spec) => spec.payable(),
        }
    }
    pub fn mutates(&self) -> bool {
        match self {
            ConstructorOrMessageSpec::Constructor(_spec) => true,
            ConstructorOrMessageSpec::Message(spec) => spec.mutates(),
        }
    }
}
