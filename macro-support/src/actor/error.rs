#[derive(thiserror::Error, Debug)]
/// Errors related to actor's implementation parsing.
pub enum Error {
    /// This error is thrown when procedural macro is not used on an implementation
    #[error("#[fvm_actor] should be used with an implementation.")]
    ExpectedImplementation,
    /// This error is thrown when an implementation is declared with generics
    #[error("implementation with #[fvm_actor] cannot have lifetime or type parameters.")]
    GenericsOnInterface,
    /// This error is thrown when the implementation for the actor interface is not for an expected structure
    #[error("expected implementation for type with no leading colon, 1 path segment, and no angle bracketed or parenthesized path arguments with #[fvm_actor]")]
    UnexpectedImplementationType,
}
