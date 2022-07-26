//! The `macro-support` is responsible for the logic coordination behind the Filecoin Virtual Machine
//! macros

extern crate fvm_rs_sdk_backend as backend;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use crate::payload::attrs::PayloadAttrs;
use backend::Diagnostic;
use proc_macro2::TokenStream;

use crate::state::attrs::StateAttrs;

mod actor;
mod export;
mod payload;
mod state;
mod utils;

pub enum MacroType {
    State,
    Actor,
    Payload,
}

/// Takes the parsed input from a procedural macro and returns the generated bindings
pub fn expand(
    macro_type: MacroType,
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, Diagnostic> {
    use crate::utils::MacroParse;
    use backend::TryToTokens;

    let item = syn::parse2::<syn::Item>(input)?;

    let mut tokens = TokenStream::new();
    let mut program = backend::ast::Program::default();

    // First step is to parse the `TokenStream` to copy source tokens & generate custom AST structures
    // for the codegen step
    match macro_type {
        MacroType::State => {
            let attrs: StateAttrs = syn::parse2(attr)?;

            item.macro_parse(&mut program, (Some(attrs), &mut tokens))?;
        }
        MacroType::Payload => {
            let attrs: PayloadAttrs = syn::parse2(attr)?;

            item.macro_parse(&mut program, (Some(attrs), &mut tokens))?;
        }
        _ => {
            item.macro_parse(&mut program, &mut tokens)?;
        }
    }

    // Second step is to generate code custom tokens based on custom AST structures & append it to
    // the `TokenStream`
    program.try_to_tokens(&mut tokens)?;

    Ok(tokens)
}
