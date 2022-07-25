#[derive(thiserror::Error, Debug)]
/// Errors related to an actor's interface.
pub enum Error {
    /// This error is thrown when the specified attribute is not handled
    #[error("unknown attribute '{0}'")]
    UnknownAttribute(String),
    /// This error is thrown when the specified dispatch method is not handled
    #[error("unknown dispatch method '{0}'")]
    UnkownDispatchMethod(String),
    /// This error is thrown when the dispatch method is not a literal string
    #[error("invalid codec format, {0}")]
    InvalidDispatchMethodFormat(String),
}
