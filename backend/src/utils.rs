use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream, Result};

// AnyIdent is a wrapper around Ident to be able to implement Parse trait
pub(crate) struct AnyIdent(pub(crate) Ident);

impl Parse for AnyIdent {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|cursor| match cursor.ident() {
            Some((ident, remaining)) => Ok((AnyIdent(ident), remaining)),
            None => Err(cursor.error("expected an identifier")),
        })
    }
}
