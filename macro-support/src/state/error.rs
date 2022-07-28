#[derive(thiserror::Error, Debug)]
/// Errors related to actor's state structure parsing.
pub enum Error {
    /// This error is thrown when procedural macro is not used on a structure
    #[error("#[fvm_state] should be used with a structure.")]
    ExpectedStructure,
}
