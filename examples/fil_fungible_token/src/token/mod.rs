pub mod state;
pub mod types;

use fvm_rs_sdk::shared::address::Address;
use fvm_rs_sdk::shared::ActorID;

use fvm_rs_sdk::shared::error::ExitCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("invalid negative: {0}")]
    InvalidNegative(String),
    #[error(
        "expected {0:?} to be a resolvable id address but none found when attempting to resolve"
    )]
    InvalidIdAddress(Address),
    #[error("caller is not actor owner: found {0}, expected: {0}")]
    CallerNotOwner(ActorID, ActorID),
}

impl TokenError {
    pub fn abort(&self) -> ! {
        fvm_rs_sdk::syscall::vm::abort(
            ExitCode::USR_UNSPECIFIED.value(),
            Some(&format!("{}", &self)),
        )
    }
}
