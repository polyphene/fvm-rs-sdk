//! The `macro-support` is responsible for the logic coordination behind the Filecoin Virtual Machine
//! macros

extern crate proc_macro2;
extern crate quote;
extern crate syn;

use backend::Diagnostic;
use proc_macro2::TokenStream;

mod fvm_state_parser;

/// Takes the parsed input from a `#[fvm_state]` macro and returns the generated bindings
pub fn expand_state(_attr: TokenStream, input: TokenStream) -> Result<TokenStream, Diagnostic> {
    todo!("implement")
}
