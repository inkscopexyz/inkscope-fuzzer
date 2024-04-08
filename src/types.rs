use drink::{
    frame_system,
    runtime::{
        AccountIdFor,
        HashFor,
    },
    BalanceOf,
    MinimalRuntime,
};

// TODO: add this to drink/runtime.rs
pub type HashingFor<R> = <R as frame_system::Config>::Hashing;

// This defines all the configurable types based on the current runtime: MinimalRuntime
pub type Balance = BalanceOf<MinimalRuntime>;
pub type AccountId = AccountIdFor<MinimalRuntime>;
// pub type Hash = HashFor<MinimalRuntime>;
pub type CodeHash = HashFor<MinimalRuntime>;
pub type Hashing = HashingFor<MinimalRuntime>;
pub type TraceHash = u64;
