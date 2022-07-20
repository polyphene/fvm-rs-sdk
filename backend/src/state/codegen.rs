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
        for s in self.state_structs.iter() {
            s.to_tokens(into);
        }

        Ok(())
    }
}

impl ToTokens for ast::StateStruct {
    fn to_tokens(&self, into: &mut TokenStream) {
        // Add derive for serialize & deserialize
        *into = (quote! {
            #[derive(fvm_rs_sdk::encoding::tuple::Serialize_tuple, fvm_rs_sdk::encoding::tuple::Deserialize_tuple)]
            #[serde( crate = "fvm_rs_sdk::encoding::serde")]
            #into
        })
            .to_token_stream();

        self.generate_state_interface().to_tokens(into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::attrs::Codec::DagCbor;

    #[test]
    fn basic_struct() {
        // Instantiate expected result
        let mut expected_final_stream = TokenStream::new();

        (quote! {
            #[derive(fvm_rs_sdk::encoding::tuple::Serialize_tuple, fvm_rs_sdk::encoding::tuple::Deserialize_tuple)]
            #[serde( crate = "fvm_rs_sdk::serde")]
            pub struct MockStruct {
                pub count: u64
            }

            impl fvm_rs_sdk::state::StateObject for MockStruct {
                fn load() -> Self {
                    use fvm_rs_sdk::state::Blockstore;

                    // First, load the current state root.
                    let root = match fvm_rs_sdk::syscall::sself::root() {
                        Ok(root) => root,
                        Err(err) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                            Some(format!("failed to get root: {:?}", err).as_str()),
                        ),
                    };

                    // Get state's bytes
                    match fvm_rs_sdk::state::cbor::CborBlockstore.get(&root) {
                        // State fetched, convert byte to actual struct
                        Ok(Some(state_bytes)) => match fvm_rs_sdk::encoding::from_slice(&state_bytes) {
                            Ok(state) => state,
                            Err(err) => fvm_rs_sdk::syscall::vm::abort(
                                fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
                                Some(format!("failed to deserialize state: {}", err).as_str()),
                            )
                        },
                        // No state
                        Ok(None) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                            Some("state does not exist"),
                        ),
                        // Fetching failed
                        Err(err) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                            Some(format!("failed to get state: {}", err).as_str()),
                        ),
                    }
                }

                fn save(&self) -> fvm_rs_sdk::cid::Cid {
                    use fvm_rs_sdk::state::Blockstore;

                    // Serialize state
                    let serialized = match fvm_rs_sdk::encoding::to_vec(self) {
                        Ok(s) => s,
                        Err(err) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
                            Some(format!("failed to serialize state: {:?}", err).as_str()),
                        ),
                    };

                    // Put state
                    let cid = match fvm_rs_sdk::state::cbor::CborBlockstore.put(
                        fvm_rs_sdk::cid::Code::Blake2b256.into(),
                        &fvm_rs_sdk::state::Block {
                            codec: fvm_rs_sdk::encoding::DAG_CBOR,
                            data: serialized.as_slice()
                        }
                    ) {
                        Ok(cid) => cid,
                        Err(err) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
                            Some(format!("failed to store initial state: {:}", err).as_str()),
                        ),
                    };
                    if let Err(err) = fvm_rs_sdk::syscall::sself::set_root(&cid) {
                        fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                            Some(format!("failed to set root cid: {:}", err).as_str()),
                        );
                    }
                    cid
                }
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

                    fields.push(ast::StateStructField {
                        rust_name: member,
                        name,
                        struct_name: s.ident.clone(),
                        ty: field.ty.clone(),
                    });
                }

                let ast_struct = ast::StateStruct {
                    rust_name: s.ident.clone(),
                    name: s.ident.to_string(),
                    fields,
                    codec: DagCbor,
                };

                // Create ast::Program
                let program = ast::Program {
                    state_structs: vec![ast_struct],
                };

                program.try_to_tokens(&mut token_stream).unwrap();

                assert_eq!(token_stream.to_string(), expected_final_stream.to_string());
            }
            _ => unreachable!(),
        }
    }
}
