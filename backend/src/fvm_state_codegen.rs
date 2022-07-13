//! Codegen has the logic of code generation for our actor through the `#[fvm_state]` macro.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{ast, Diagnostic};

/// A trait for converting AST structs into Tokens and adding them to a TokenStream,
/// or providing a diagnostic if conversion fails.
pub trait TryToTokens {
    /// Attempt to convert a `Self` into tokens and add it to the `TokenStream`
    fn try_to_tokens(&self, into: &mut TokenStream) -> Result<(), Diagnostic>;

    /// Attempt to convert a `Self` into a new `TokenStream`
    fn try_to_token_stream(&self) -> Result<TokenStream, Diagnostic> {
        let mut tokens = TokenStream::new();
        self.try_to_tokens(&mut tokens)?;
        Ok(tokens)
    }
}

impl TryToTokens for ast::Program {
    // Generate wrappers for all the items that we've found
    fn try_to_tokens(&self, into: &mut TokenStream) -> Result<(), Diagnostic> {
        // Handling tagged structures
        for s in self.structs.iter() {
            s.to_tokens(into);
        }

        Ok(())
    }
}

impl ToTokens for ast::Struct {
    fn to_tokens(&self, into: &mut TokenStream) {
        // Add derive for serialize & deserialize
        *into = (quote! {
            #[derive(fvm_rs_sdk::serde_tuple::Serialize_tuple, fvm_rs_sdk::serde_tuple::Deserialize_tuple)]
            #[serde( crate = "fvm_rs_sdk::serde")]
            #into
        })
            .to_token_stream();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_struct() {
        // Instantiate expected result
        let mut expected_final_stream = TokenStream::new();

        (quote! {
            #[derive(fvm_rs_sdk::serde_tuple::Serialize_tuple, fvm_rs_sdk::serde_tuple::Deserialize_tuple)]
            #[serde( crate = "fvm_rs_sdk::serde")]tatus

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
            syn::Item::Struct(mut s) => {
                // Generate ast::Struct
                let mut fields = Vec::new();
                for field in s.fields.iter_mut() {
                    // Derive field name from ident
                    let (name, member) = match &field.ident {
                        Some(ident) => (ident.to_string(), syn::Member::Named(ident.clone())),
                        _ => unreachable!(),
                    };

                    fields.push(ast::StructField {
                        rust_name: member,
                        name,
                        struct_name: s.ident.clone(),
                        ty: field.ty.clone(),
                    });
                }

                let ast_struct = ast::Struct {
                    rust_name: s.ident.clone(),
                    name: s.ident.to_string(),
                    fields,
                };

                // Create ast::Program
                let program = ast::Program {
                    structs: vec![ast_struct],
                };

                program.try_to_tokens(&mut token_stream).unwrap();

                assert_eq!(token_stream.to_string(), expected_final_stream.to_string());
            }
            _ => unreachable!(),
        }
    }
}
