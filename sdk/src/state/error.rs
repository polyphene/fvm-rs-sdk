use cid::Cid;
use fvm_shared::error::ErrorNumber;

#[derive(thiserror::Error, Debug)]
/// Errors related to an actor's state.
pub enum Error {
    /// This error is thrown when trying to load data from an invalid CID
    #[error("get failed with {0:?} on CID '{1}'")]
    InvalidCid(ErrorNumber, Cid),
    /// This error is thrown when trying to put a block with mismatched content and CID
    #[error("put block with cid {0} but has cid {1}")]
    MismatchedCid(Cid, Cid),
    /// This error is thrown when a put fails
    #[error("put failed with {0:?}")]
    PutFailed(ErrorNumber),
}
