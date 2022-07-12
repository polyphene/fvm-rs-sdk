//! The `macro` crate is the entry point for the FVM SDK procedural macros

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn fvm_state(attr: TokenStream, input: TokenStream) -> TokenStream {
    //TODO Implement
    input
}
