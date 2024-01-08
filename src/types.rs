use crate::ext::*;
use crate::flags::*;
use codec::{Decode, Encode};
use wasmi::Memory;

// These are the types and constants that are used by the specific runtime where pallet contract is implemented
// Although unlikely, different parachains may be configured with different types and constants
pub type AccountId = [u8; 32];
pub type Hash = [u8; 32];
pub type Balance = u128;
pub type CodeType = Vec<u8>;
pub type AllowDeprecatedInterface = bool;
pub type AllowUnstableInterface = bool;
pub type Determinism = bool; // If true the execution should be deterministic and hence no indeterministic instructions are allowed.
pub type Weight = u64;
pub type Key = [u8; 32];
pub type MomentOf = u64;
pub type SeedOf = Hash;
pub type BlockNumberFor = u64;
pub type TopicOf = Hash;

/// The number of bytes of the module-specific `error` field defined in [`ModuleError`].
/// In FRAME, this is the maximum encoded size of a pallet error type.
pub const MAX_MODULE_ERROR_ENCODED_SIZE: usize = 4;

/// Reason why a pallet call failed.
#[derive(Eq, Clone, Copy, Encode, Decode, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModuleError {
    /// Module index, matching the metadata module index.
    pub index: u8,
    /// Module specific error value.
    pub error: [u8; MAX_MODULE_ERROR_ENCODED_SIZE],
    /// Optional error message.
    #[codec(skip)]
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub message: Option<&'static str>,
}
impl PartialEq for ModuleError {
    fn eq(&self, other: &Self) -> bool {
        (self.index == other.index) && (self.error == other.error)
    }
}

/// Description of what went wrong when trying to complete an operation on a token.
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TokenError {
    /// Funds are unavailable.
    FundsUnavailable,
    /// Some part of the balance gives the only provider reference to the account and thus cannot
    /// be (re)moved.
    OnlyProvider,
    /// Account cannot exist with the funds that would be given.
    BelowMinimum,
    /// Account cannot be created.
    CannotCreate,
    /// The asset in question is unknown.
    UnknownAsset,
    /// Funds exist but are frozen.
    Frozen,
    /// Operation is not supported by the asset.
    Unsupported,
    /// Account cannot be created for a held balance.
    CannotCreateHold,
    /// Withdrawal would cause unwanted loss of account.
    NotExpendable,
    /// Account cannot receive the assets.
    Blocked,
}

/// Arithmetic errors.
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ArithmeticError {
    /// Underflow.
    Underflow,
    /// Overflow.
    Overflow,
    /// Division by zero.
    DivisionByZero,
}

/// Errors related to transactional storage layers.
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TransactionalError {
    /// Too many transactional layers have been spawned.
    LimitReached,
    /// A transactional layer was expected, but does not exist.
    NoLayer,
}

/// Reason why a dispatch call failed.
//#[derive(Eq, Clone, Copy, Encode, Decode, Debug, TypeInfo, PartialEq, MaxEncodedLen)]
#[derive(Eq, Clone, Copy, Encode, Decode, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DispatchError {
    /// Some error occurred.
    Other(
        #[codec(skip)]
        #[cfg_attr(feature = "serde", serde(skip_deserializing))]
        &'static str,
    ),
    /// Failed to lookup some data.
    CannotLookup,
    /// A bad origin.
    BadOrigin,
    /// A custom error in a module.
    Module(ModuleError),
    /// At least one consumer is remaining so the account cannot be destroyed.
    ConsumerRemaining,
    /// There are no providers so the account cannot be created.
    NoProviders,
    /// There are too many consumers so the account cannot be created.
    TooManyConsumers,
    /// An error to do with tokens.
    Token(TokenError),
    /// An arithmetic error.
    Arithmetic(ArithmeticError),
    /// The number of transactional layers has been reached, or we are not in a transactional
    /// layer.
    Transactional(TransactionalError),
    /// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
    Exhausted,
    /// The state is corrupt; this is generally not going to fix itself.
    Corruption,
    /// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
    Unavailable,
    /// Root origin is not allowed.
    RootNotAllowed,
}

/// Information about what happened to the pre-existing value when calling [`ContractInfo::write`].
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum WriteOutcome {
    /// No value existed at the specified key.
    New,
    /// A value of the returned length was overwritten.
    Overwritten(u32),
    /// The returned value was taken out of storage before being overwritten.
    ///
    /// This is only returned when specifically requested because it causes additional work
    /// depending on the size of the pre-existing value. When not requested [`Self::Overwritten`]
    /// is returned instead.
    Taken(Vec<u8>),
}

/// Result of a `Dispatchable` which contains the `DispatchResult` and additional information about
/// the `Dispatchable` that is only known post dispatch.
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode)]
pub struct DispatchErrorWithPostInfo<Info>
where
    Info: Eq + PartialEq + Clone + Copy + Encode + Decode,
{
    /// Additional information about the `Dispatchable` which is only known post dispatch.
    pub post_info: Info,
    /// The actual `DispatchResult` indicating whether the dispatch was successful.
    pub error: DispatchError,
}

pub type DispatchResult = Result<(), DispatchError>;

#[derive(Clone, Encode, Decode, PartialEq)]
pub enum Origin {
    Root,
    Signed(AccountId),
}
/// Output of a contract call or instantiation which ran to completion.
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
pub struct ExecReturnValue {
    /// Flags passed along by `seal_return`. Empty when `seal_return` was never called.
    pub flags: ReturnFlags,
    /// Buffer passed along by `seal_return`. Empty when `seal_return` was never called.
    pub data: Vec<u8>,
}

impl ExecReturnValue {
    /// The contract did revert all storage changes.
    pub fn did_revert(&self) -> bool {
        self.flags.contains(ReturnFlags::REVERT)
    }
}

/// Origin of the error.
///
/// Call or instantiate both called into other contracts and pass through errors happening
/// in those to the caller. This enum is for the caller to distinguish whether the error
/// happened during the execution of the callee or in the current execution context.
#[derive(Copy, Clone, PartialEq, Eq, Debug, codec::Decode, codec::Encode)]
pub enum ErrorOrigin {
    /// Caller error origin.
    ///
    /// The error happened in the current execution context rather than in the one
    /// of the contract that is called into.
    Caller,
    /// The error happened during execution of the called contract.
    Callee,
}

/// Error returned by contract execution.
#[derive(Copy, Clone, PartialEq, Eq, Debug, codec::Decode, codec::Encode)]
pub struct ExecError {
    /// The reason why the execution failed.
    pub error: DispatchError,
    /// Origin of the error.
    pub origin: ErrorOrigin,
}

impl<T: Into<DispatchError>> From<T> for ExecError {
    fn from(error: T) -> Self {
        Self {
            error: error.into(),
            origin: ErrorOrigin::Caller,
        }
    }
}

pub type ExecResult = Result<ExecReturnValue, ExecError>;

/// Can only be used for one call.
pub struct Runtime<'a, E: Ext + 'a> {
    ext: &'a mut E,
    input_data: Option<Vec<u8>>,
    memory: Option<Memory>,
}

impl<'a, E: Ext + 'a> Runtime<'a, E> {
    pub fn new(ext: &'a mut E, input_data: Vec<u8>) -> Self {
        Runtime {
            ext,
            input_data: Some(input_data),
            memory: None,
        }
    }
    pub fn memory(&self) -> Option<Memory> {
        self.memory
    }

    pub fn set_memory(&mut self, memory: Memory) {
        self.memory = Some(memory);
    }

    /// Get a mutable reference to the inner `Ext`.
    ///
    /// This is mainly for the chain extension to have access to the environment the
    /// contract is executing in.
    pub fn ext(&mut self) -> &mut E {
        self.ext
    }
}
