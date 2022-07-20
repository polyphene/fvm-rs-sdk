//! The `macro` crate is the entry point for the FVM SDK procedural macros

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn fvm_state(attr: TokenStream, input: TokenStream) -> TokenStream {
    match fvm_rs_sdk_macro_support::expand_state(attr.into(), input.into()) {
        Ok(tokens) => tokens.into(),
        Err(diagnostic) => (quote! { #diagnostic }).into(),
    }
}
