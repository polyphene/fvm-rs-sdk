#[derive(thiserror::Error, Debug)]
/// Errors related to exported method parsing.
pub enum Error {
    /// This error is thrown when a method is exported but has no binding
    #[error("binding should be specified on method '{0}'")]
    MissingBinding(String),
    /// This error is thrown when an entry point is declared with generics
    #[error("'{0}' can not be used as an entry point. Methods with #[fvm_export] cannot have lifetime or type parameters.")]
    GenericsOnEntryPoint(String),
    /// This error is thrown when an argument in a method has an unexpected type
    #[error("{0}, '{1}', can not be used as a type for an entry point argument.")]
    UnexpectedArgType(String, String),
    /// This error is thrown when an argument has a type that can not be interpreted
    #[error("'{0}' can not be interpreted and thus can not be used as a type for an entry point argument.")]
    UnhandledType(String),
    /// This error is thrown when an argument is of receiver type at an unexpected position
    #[error("'self' should only be used as first argument for an entry point argument.")]
    UnexpectedArgReceiver,
    /// This error is thrown when the pattern for argument is not a biding to a new variable
    #[error("expected binding to variable when parsing method arguments.")]
    ExpectedBindingToNewVariable,
}
