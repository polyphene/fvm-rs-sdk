#[derive(thiserror::Error, Debug)]
/// Errors related to actor's implementation parsing.
pub enum Error {
    /// This error is thrown when an entry point is declared with generics
    #[error("implementation with #[fvm_actor] cannot have lifetime or type parameters.")]
    GenericsOnInterface,
    /// This error is thrown when the implementation for the actor interface is not for an expected structure
    #[error("expected implementation for type with no leading colon, 1 path segment, and no angle bracketed or parenthesized path arguments with #[fvm_actor]")]
    UnexpectedImplementationType,
}
