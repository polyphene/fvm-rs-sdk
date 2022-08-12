use crate::Diagnostic;
use proc_macro2::{Ident, TokenStream};
use syn::parse::{Parse, ParseStream, Result as SynResult};

// AnyIdent is a wrapper around Ident to be able to implement Parse trait
pub(crate) struct AnyIdent(pub(crate) Ident);

impl Parse for AnyIdent {
    fn parse(input: ParseStream) -> SynResult<Self> {
        input.step(|cursor| match cursor.ident() {
            Some((ident, remaining)) => Ok((AnyIdent(ident), remaining)),
            None => Err(cursor.error("expected an identifier")),
        })
    }
}

/// A trait for converting AST structs into Tokens and adding them to a TokenStream,
/// or providing a diagnostic if conversion fails.
pub trait TryToTokens {
    /// Attempt to convert a `Self` into tokens and add it to the `TokenStream`
    fn try_to_tokens(&self, into: &mut TokenStream) -> Result<(), Diagnostic>;

    /// Attempt to convert a `Self` into a new `TokenStream`
    fn try_to_token_stream(&self) -> Result<TokenStream, Diagnostic> {
        let mut tokens = TokenStream::new();
        self.try_to_tokens(&mut tokens)?;
        Ok(tokens)
    }
}
