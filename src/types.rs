// These are the types and constants that are used by the specific runtime where pallet contract is implemented
// Although unlikely, different parachains may be configured with different types and constants
pub type AccountId = [u8; 32];
pub type Hash = [u8; 32];
pub type Balance = u128;
pub type CodeType = Vec<u8>;
pub type AllowDeprecatedInterface = bool;
pub type AllowUnstableInterface = bool;
pub type Determinism = bool; // If true the execution should be deterministic and hence no indeterministic instructions are allowed.
