//! The `macro-support` is responsible for the logic coordination behind the Filecoin Virtual Machine
//! macros

extern crate proc_macro2;
extern crate quote;
extern crate syn;
#[macro_use]
extern crate fvm_rs_sdk_backend as backend;

use backend::Diagnostic;
use proc_macro2::TokenStream;

mod state;

/// Takes the parsed input from a `#[fvm_state]` macro and returns the generated bindings
pub fn expand_state(attr: TokenStream, input: TokenStream) -> Result<TokenStream, Diagnostic> {
    use backend::TryStateToTokens;

    use crate::state::parser::MacroParse;

    let item = syn::parse2::<syn::Item>(input)?;
    let attrs = syn::parse2(attr)?;

    let mut tokens = TokenStream::new();
    let mut program = backend::ast::Program::default();

    // First step is to parse the `TokenStream` to copy source tokens & generate custom AST structures
    // for the codegen step
    item.macro_parse(&mut program, (Some(attrs), &mut tokens))?;

    // Second step is to generate code custom tokens based on custom AST structures & append it to
    // the `TokenStream`
    program.try_to_tokens(&mut tokens)?;

    Ok(tokens)
}
