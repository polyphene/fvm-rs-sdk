use cid::Cid;
use fvm_shared::error::ErrorNumber;

#[derive(thiserror::Error, Debug)]
/// Errors related to an actor's state.
pub(crate) enum Error {
    /// This error is thrown when trying to load data from an invalid CID
    #[error("get failed with {:?} on CID '{}'")]
    InvalidCid(ErrorNumber, Cid),
    /// This error is thrown when trying to put a block with mismatched content and CID
    #[error("put block with cid {} but has cid {}")]
    MismatchedCid(Cid, Cid),
    /// This error is thrown when a put fails
    #[error("put failed with {:?}")]
    PutFailed(ErrorNumber),
}
