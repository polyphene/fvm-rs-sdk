//! Codegen has the logic of code generation for our actor through the `#[fvm_state]` macro.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::ast;

impl ToTokens for ast::ActorImplementation {
    fn to_tokens(&self, into: &mut TokenStream) {
        // TODO

        quote!().to_tokens(into)
    }
}

#[cfg(test)]
mod tests {}
