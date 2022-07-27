#[derive(thiserror::Error, Debug)]
/// Errors related to exported method parsing.
pub enum Error {
    /// This error is thrown when a method is exported but has no binding
    #[error("binding should be specified on method '{0}'")]
    MissingBinding(String),
    /// This error is thrown when the dispatch method and a binding does not match
    #[error("binding for '{0}' does not match dispatch method specified. Expected {1}.")]
    MismatchedDispatchBinding(String, String),
}
