#[derive(thiserror::Error, Debug)]
/// Errors related to an actor's interface.
pub enum Error {
    /// This error is thrown when the specified attribute is not handled
    #[error("unknown attribute '{0}'")]
    UnknownAttribute(String),
    /// This error is thrown when the value provided for the binding is not one we can handle
    #[error("invalid binding value")]
    InvalidBindingValue,
    /// This error is thrown when the numeric entry point value can not be parsed as u64
    #[error("invalid codec format, {0}")]
    InvalidNumericValue(String),
}
