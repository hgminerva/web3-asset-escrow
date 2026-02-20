
use scale::{Decode, Encode};
use ink::env::Error as EnvError;

/// Error Messages
#[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// Bad origin error, e.g., wrong caller
    BadOrigin,
    /// Escrow is close
    EscrowIsClose,
    /// Escrow account not found
    EscrowAccountNotFound,
    /// One escrow per account only
    EscrowAccountDuplicate,
    /// Maximum escrow account has been reached
    EscrowAccountMax,
}  

/// Runtime call execution error
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum RuntimeError {
    /// Failed to dispatch a runtime call.
    CallRuntimeFailed,
}

/// Unified contract error type.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    /// Internal errors.
    Internal(Error),
    /// Runtime call errors.
    Runtime(RuntimeError),
}

// Error conversions for convenience.
impl From<Error> for ContractError {
    fn from(err: Error) -> Self {
        Self::Internal(err)
    }
}

impl From<RuntimeError> for ContractError {
    fn from(err: RuntimeError) -> Self {
        Self::Runtime(err)
    }
}

impl From<EnvError> for RuntimeError {
    fn from(e: EnvError) -> Self {
        use ink::env::ReturnErrorCode;
        match e {
            EnvError::ReturnError(ReturnErrorCode::CallRuntimeFailed) => {
                Self::CallRuntimeFailed
            }
            _ => panic!("Unexpected error from pallet_contracts environment"),
        }
    }
}