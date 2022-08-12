#[derive(thiserror::Error, Debug)]
/// Errors related to actor's state structure parsing.
pub enum Error {
    /// This error is thrown when a state structure is declared with generics
    #[error("structure with #[fvm_state] cannot have lifetime or type parameters.")]
    GenericsOnStructure,
    /// This error is thrown when procedural macro is not used on a structure
    #[error("#[fvm_state] should be used with a structure.")]
    ExpectedStructure,
}
