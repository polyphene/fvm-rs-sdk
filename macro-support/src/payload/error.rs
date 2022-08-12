#[derive(thiserror::Error, Debug)]
/// Errors related to actor's payload structure parsing.
pub enum Error {
    /// This error is thrown when a payload structure is declared with generics
    #[error("structure with #[fvm_payload] cannot have lifetime or type parameters.")]
    GenericsOnStructure,
    /// This error is thrown when procedural macro is not used on a structure
    #[error("#[fvm_payload] should be used with a structure.")]
    ExpectedStructure,
}
