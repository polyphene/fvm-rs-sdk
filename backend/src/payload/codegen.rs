//! Codegen has the logic of code generation for our actor through the `#[fvm_payload]` macro.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::ast;

impl ToTokens for ast::PayloadStruct {
    fn to_tokens(&self, into: &mut TokenStream) {
        // Add derive for serialize & deserialize
        *into = (quote! {
            #[derive(fvm_rs_sdk::encoding::tuple::Serialize_tuple, fvm_rs_sdk::encoding::tuple::Deserialize_tuple)]
            #[serde( crate = "fvm_rs_sdk::encoding::serde")]
            #into
        })
            .to_token_stream();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::attrs::Codec::DagCbor;
    use crate::TryToTokens;

    #[test]
    fn basic_struct() {
        // Instantiate expected result
        let mut expected_final_stream = TokenStream::new();

        (quote! {
            #[derive(fvm_rs_sdk::encoding::tuple::Serialize_tuple, fvm_rs_sdk::encoding::tuple::Deserialize_tuple)]
            #[serde( crate = "fvm_rs_sdk::encoding::serde")]
            pub struct MockStruct {
                pub count: u64
            }
        })
            .to_tokens(&mut expected_final_stream);

        // Create new token stream
        let mut token_stream = TokenStream::new();

        // Add a structure to our stream
        (quote! {
            pub struct MockStruct {
                pub count: u64
            }
        })
        .to_tokens(&mut token_stream);

        // Convert token stream to syn structure
        let item = syn::parse2::<syn::Item>(token_stream.clone()).unwrap();

        match item {
            syn::Item::Struct(s) => {
                // Generate ast::Struct
                let ast_struct = ast::PayloadStruct {
                    rust_name: s.ident.to_token_stream(),
                    name: s.ident.to_string(),
                    codec: DagCbor,
                };

                // Create ast::Program
                let program = ast::Program {
                    payload_structs: vec![ast_struct],
                    actor_implementation: None,
                    state_structs: vec![],
                };

                program.try_to_tokens(&mut token_stream).unwrap();

                assert_eq!(token_stream.to_string(), expected_final_stream.to_string());
            }
            _ => unreachable!(),
        }
    }
}
