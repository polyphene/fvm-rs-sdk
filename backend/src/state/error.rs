#[derive(thiserror::Error, Debug)]
/// Errors related to an actor's state.
pub enum Error {
    /// This error is thrown when the specified attribute is not handled
    #[error("unknown attribute '{0}'")]
    UnknownAttribute(String),
    /// This error is thrown when the specified codec is not handled
    #[error("unknown codec '{0}'")]
    UnknownCodec(String),
    /// This error is thrown when the codec is not a literal string
    #[error("invalid codec format, {0}")]
    InvalidCodecFormat(String),
}
