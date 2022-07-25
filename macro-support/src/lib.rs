//! The `macro-support` is responsible for the logic coordination behind the Filecoin Virtual Machine
//! macros

extern crate proc_macro2;
extern crate quote;
extern crate syn;
#[macro_use]
extern crate fvm_rs_sdk_backend as backend;

use backend::Diagnostic;
use proc_macro2::TokenStream;

mod actor;
mod state;
mod utils;

pub enum MacroType {
    State,
    Actor,
    Export,
}

/// Takes the parsed input from a `#[fvm_state]` macro and returns the generated bindings
pub fn expand_state(
    macro_type: MacroType,
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, Diagnostic> {
    use backend::TryStateToTokens;

    let item = syn::parse2::<syn::Item>(input)?;

    let mut tokens = TokenStream::new();
    let mut program = backend::ast::Program::default();

    // First step is to parse the `TokenStream` to copy source tokens & generate custom AST structures
    // for the codegen step
    match macro_type {
        MacroType::State => {
            use crate::state::parser::MacroParse;
            let attrs = syn::parse2(attr)?;

            item.macro_parse(&mut program, (Some(attrs), &mut tokens))?;
        }
        MacroType::Actor => {
            use crate::actor::parser::MacroParse;
            let attrs = syn::parse2(attr)?;

            item.macro_parse(&mut program, (Some(attrs), &mut tokens))?;
        }
        MacroType::Export => {
            todo!()
        }
    }

    // Second step is to generate code custom tokens based on custom AST structures & append it to
    // the `TokenStream`
    program.try_to_tokens(&mut tokens)?;

    Ok(tokens)
}
