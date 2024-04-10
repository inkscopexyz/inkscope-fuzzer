

use ink_sandbox::{
    frame_system, pallet_balances, macros::DefaultSandboxRuntime
};

pub type BalanceOf<R> = <R as pallet_balances::Config>::Balance;
pub type HashingFor<R> = <R as frame_system::Config>::Hashing;
pub type AccountIdFor<R> = <R as frame_system::Config>::AccountId;
pub type HashFor<R> = <R as frame_system::Config>::Hash;

// This defines all the configurable types based on the current runtime: MinimalRuntime
pub type Balance = BalanceOf<DefaultSandboxRuntime>;
pub type AccountId = AccountIdFor<DefaultSandboxRuntime>;
pub type CodeHash = HashFor<DefaultSandboxRuntime>;
pub type Hashing = HashingFor<DefaultSandboxRuntime>;
pub type TraceHash = u64;
