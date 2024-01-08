use crate::types::*;

/// An interface that provides access to the external environment in which the
/// smart-contract is executed.
///
/// This interface is specialized to an account of the executing code, so all
/// operations are implicitly performed on that account.
///
/// # Note
///
/// This trait is sealed and cannot be implemented by downstream crates.
pub trait Ext {
    /// Call (possibly transferring some amount of funds) into the specified account.
    ///
    /// Returns the code size of the called contract.
    fn call(
        &mut self,
        gas_limit: Weight,
        deposit_limit: Balance,
        to: AccountId,
        value: Balance,
        input_data: Vec<u8>,
        allows_reentry: bool,
    ) -> Result<ExecReturnValue, ExecError>;

    /// Execute code in the current frame.
    ///
    /// Returns the code size of the called contract.
    fn delegate_call(
        &mut self,
        code: Hash,
        input_data: Vec<u8>,
    ) -> Result<ExecReturnValue, ExecError>;

    /// Instantiate a contract from the given code.
    ///
    /// Returns the original code size of the called contract.
    /// The newly created account will be associated with `code`. `value` specifies the amount of
    /// value transferred from the caller to the newly created account.
    fn instantiate(
        &mut self,
        gas_limit: Weight,
        deposit_limit: Balance,
        code: Hash,
        value: Balance,
        input_data: Vec<u8>,
        salt: &[u8],
    ) -> Result<(AccountId, ExecReturnValue), ExecError>;

    /// Transfer all funds to `beneficiary` and delete the contract.
    ///
    /// Since this function removes the self contract eagerly, if succeeded, no further actions
    /// should be performed on this `Ext` instance.
    ///
    /// This function will fail if the same contract is present on the contract
    /// call stack.
    fn terminate(&mut self, beneficiary: &AccountId) -> Result<(), DispatchError>;

    /// Transfer some amount of funds into the specified account.
    fn transfer(&mut self, to: &AccountId, value: Balance) -> DispatchResult;

    /// Returns the storage entry of the executing account by the given `key`.
    ///
    /// Returns `None` if the `key` wasn't previously set by `set_storage` or
    /// was deleted.
    fn get_storage(&mut self, key: &Key) -> Option<Vec<u8>>;

    /// Returns `Some(len)` (in bytes) if a storage item exists at `key`.
    ///
    /// Returns `None` if the `key` wasn't previously set by `set_storage` or
    /// was deleted.
    fn get_storage_size(&mut self, key: &Key) -> Option<u32>;

    /// Sets the storage entry by the given key to the specified value. If `value` is `None` then
    /// the storage entry is deleted.
    fn set_storage(
        &mut self,
        key: &Key,
        value: Option<Vec<u8>>,
        take_old: bool,
    ) -> Result<WriteOutcome, DispatchError>;

    /// Returns the caller.
    fn caller(&self) -> Origin;

    /// Check if a contract lives at the specified `address`.
    fn is_contract(&self, address: &AccountId) -> bool;

    /// Returns the code hash of the contract for the given `address`.
    ///
    /// Returns `None` if the `address` does not belong to a contract.
    fn code_hash(&self, address: &AccountId) -> Option<Hash>;

    /// Returns the code hash of the contract being executed.
    fn own_code_hash(&mut self) -> &Hash;

    /// Check if the caller of the current contract is the origin of the whole call stack.
    ///
    /// This can be checked with `is_contract(self.caller())` as well.
    /// However, this function does not require any storage lookup and therefore uses less weight.
    fn caller_is_origin(&self) -> bool;

    /// Check if the caller is origin, and this origin is root.
    fn caller_is_root(&self) -> bool;

    /// Returns a reference to the account id of the current contract.
    fn address(&self) -> &AccountId;

    /// Returns the balance of the current contract.
    ///
    /// The `value_transferred` is already added.
    fn balance(&self) -> Balance;

    /// Returns the value transferred along with this call.
    fn value_transferred(&self) -> Balance;

    /// Returns a reference to the timestamp of the current block
    fn now(&self) -> &MomentOf;

    /// Returns the minimum balance that is required for creating an account.
    fn minimum_balance(&self) -> Balance;

    /// Returns a random number for the current block with the given subject.
    fn random(&self, subject: &[u8]) -> (SeedOf, BlockNumberFor);

    /// Deposit an event with the given topics.
    ///
    /// There should not be any duplicates in `topics`.
    fn deposit_event(&mut self, topics: Vec<TopicOf>, data: Vec<u8>);

    /// Returns the current block number.
    fn block_number(&self) -> BlockNumberFor;

    /// Returns the maximum allowed size of a storage item.
    fn max_value_size(&self) -> u32;

    /// Returns the price for the specified amount of weight.
    fn get_weight_price(&self, weight: Weight) -> Balance;

    /// Get a reference to the schedule used by the current call.
    //fn schedule(&self) -> &Schedule<Self::T>; //TODO: Check if this can be removed

    /// Get an immutable reference to the nested gas meter.
    //fn gas_meter(&self) -> &GasMeter<Self::T>; //TODO: Check if this can be removed

    /// Get a mutable reference to the nested gas meter.
    //fn gas_meter_mut(&mut self) -> &mut GasMeter<Self::T>;//TODO: Check if this can be removed

    /// Charges `diff` from the meter.
    //fn charge_storage(&mut self, diff: &Diff);//TODO: Check if this can be removed

    /// Append a string to the debug buffer.
    ///
    /// It is added as-is without any additional new line.
    ///
    /// This is a no-op if debug message recording is disabled which is always the case
    /// when the code is executing on-chain.
    ///
    /// Returns `true` if debug message recording is enabled. Otherwise `false` is returned.
    fn append_debug_buffer(&mut self, msg: &str) -> bool;

    /// Call some dispatchable and return the result.
    //fn call_runtime(&self, call: <Self::T as Config>::RuntimeCall) -> DispatchResultWithPostInfo; //TODO: Check if this can be removed

    /// Recovers ECDSA compressed public key based on signature and message hash.
    fn ecdsa_recover(&self, signature: &[u8; 65], message_hash: &[u8; 32]) -> Result<[u8; 33], ()>;

    /// Verify a sr25519 signature.
    fn sr25519_verify(&self, signature: &[u8; 64], message: &[u8], pub_key: &[u8; 32]) -> bool;

    /// Returns Ethereum address from the ECDSA compressed public key.
    fn ecdsa_to_eth_address(&self, pk: &[u8; 33]) -> Result<[u8; 20], ()>;

    /// Tests sometimes need to modify and inspect the contract info directly.
    // #[cfg(test)]
    // fn contract_info(&mut self) -> &mut ContractInfo<Self::T>;//TODO: Check if this can be removed

    /// Sets new code hash for existing contract.
    fn set_code_hash(&mut self, hash: Hash) -> Result<(), DispatchError>;

    /// Returns the number of times the currently executing contract exists on the call stack in
    /// addition to the calling instance. A value of 0 means no reentrancy.
    fn reentrance_count(&self) -> u32;

    /// Returns the number of times the specified contract exists on the call stack. Delegated calls
    /// are not calculated as separate entrance.
    /// A value of 0 means it does not exist on the call stack.
    fn account_reentrance_count(&self, account_id: &AccountId) -> u32;

    /// Returns a nonce that is incremented for every instantiated contract.
    fn nonce(&mut self) -> u64;

    /// Increment the reference count of a of a stored code by one.
    ///
    /// # Errors
    ///
    /// [`Error::CodeNotFound`] is returned if no stored code found having the specified
    /// `code_hash`.
    fn increment_refcount(code_hash: Hash) -> Result<(), DispatchError>;

    /// Decrement the reference count of a stored code by one.
    ///
    /// # Note
    ///
    /// A contract whose reference count dropped to zero isn't automatically removed. A
    /// `remove_code` transaction must be submitted by the original uploader to do so.
    fn decrement_refcount(code_hash: Hash);

    /// Adds a delegate dependency to [`ContractInfo`]'s `delegate_dependencies` field.
    ///
    /// This ensures that the delegated contract is not removed while it is still in use. It
    /// increases the reference count of the code hash and charges a fraction (see
    /// [`Config::CodeHashLockupDepositPercent`]) of the code deposit.
    ///
    /// # Errors
    ///
    /// - [`Error::<T>::MaxDelegateDependenciesReached`]
    /// - [`Error::<T>::CannotAddSelfAsDelegateDependency`]
    /// - [`Error::<T>::DelegateDependencyAlreadyExists`]
    fn add_delegate_dependency(&mut self, code_hash: Hash) -> Result<(), DispatchError>;

    /// Removes a delegate dependency from [`ContractInfo`]'s `delegate_dependencies` field.
    ///
    /// This is the counterpart of [`Self::add_delegate_dependency`]. It decreases the reference
    /// count and refunds the deposit that was charged by [`Self::add_delegate_dependency`].
    ///
    /// # Errors
    ///
    /// - [`Error::<T>::DelegateDependencyNotFound`]
    fn remove_delegate_dependency(&mut self, code_hash: &Hash) -> Result<(), DispatchError>;
}
