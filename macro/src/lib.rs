//! The `macro` crate is the entry point for the FVM SDK procedural macros

extern crate proc_macro;

use fvm_rs_sdk_macro_support::MacroType;
use proc_macro::TokenStream;
use quote::quote;

macro_rules! generate_proc_macro {
    ($assert_macro:ident, $macro_type:expr) => {
        #[proc_macro_attribute]
        pub fn $assert_macro(attr: TokenStream, input: TokenStream) -> TokenStream {
            match fvm_rs_sdk_macro_support::expand_state($macro_type, attr.into(), input.into()) {
                Ok(tokens) => tokens.into(),
                Err(diagnostic) => (quote! { #diagnostic }).into(),
            }
        }
    };
}

generate_proc_macro!(fvm_state, MacroType::State);
generate_proc_macro!(fvm_actor, MacroType::Actor);
