//! Codegen has the logic of code generation for our actor through the `#[fvm_state]` macro.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::ast;
use crate::state::attrs::Codec;

impl ToTokens for ast::StateStruct {
    fn to_tokens(&self, into: &mut TokenStream) {
        // Add derive for serialize & deserialize
        *into = (quote! {
            #[derive(fvm_rs_sdk::encoding::tuple::Serialize_tuple, fvm_rs_sdk::encoding::tuple::Deserialize_tuple)]
            #[serde( crate = "fvm_rs_sdk::encoding::serde")]
            #into
        })
            .to_token_stream();

        match self.codec {
            Codec::DagCbor => {
                let name = &self.rust_name;
                quote!(
                    impl fvm_rs_sdk::state::StateObject for #name {
                        fn load() -> Self {
                            use fvm_rs_sdk::encoding::CborStore;
                            // First, load the current state root.
                            let root = match fvm_rs_sdk::syscall::sself::root() {
                                Ok(root) => root,
                                Err(err) => fvm_rs_sdk::syscall::vm::abort(
                                    fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                                    Some(format!("failed to get root: {:?}", err).as_str()),
                                ),
                            };

                            // Load the actor state from the state tree.
                            match fvm_rs_sdk::state::cbor::CborBlockstore.get_cbor::<Self>(&root) {
                                Ok(Some(state)) => state,
                                Ok(None) => fvm_rs_sdk::syscall::vm::abort(
                                    fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                                    Some("state does not exist"),
                                ),
                                Err(err) => fvm_rs_sdk::syscall::vm::abort(
                                    fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                                    Some(format!("failed to get state: {}", err).as_str()),
                                ),
                            }
                        }

                        fn save(&self) -> fvm_rs_sdk::cid::Cid {
                            let serialized = match fvm_rs_sdk::encoding::to_vec(self) {
                                Ok(s) => s,
                                Err(err) => fvm_rs_sdk::syscall::vm::abort(
                                    fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
                                    Some(format!("failed to serialize state: {:?}", err).as_str()),
                                ),
                            };
                            let cid = match fvm_rs_sdk::syscall::ipld::put(
                                fvm_rs_sdk::cid::Code::Blake2b256.into(),
                                fvm_rs_sdk::state::cbor::SIZE,
                                fvm_rs_sdk::encoding::DAG_CBOR,
                                serialized.as_slice(),
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
                ).to_tokens(into);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::attrs::Codec::DagCbor;
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

            impl fvm_rs_sdk::state::StateObject for MockStruct {
                fn load() -> Self {
                    use fvm_rs_sdk::encoding::CborStore;
                    // First, load the current state root.
                    let root = match fvm_rs_sdk::syscall::sself::root() {
                        Ok(root) => root,
                        Err(err) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                            Some(format!("failed to get root: {:?}", err).as_str()),
                        ),
                    };

                    // Load the actor state from the state tree.
                    match fvm_rs_sdk::state::cbor::CborBlockstore.get_cbor::<Self>(&root) {
                        Ok(Some(state)) => state,
                        Ok(None) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                            Some("state does not exist"),
                        ),
                        Err(err) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                            Some(format!("failed to get state: {}", err).as_str()),
                        ),
                    }
                }

                fn save(&self) -> fvm_rs_sdk::cid::Cid {
                    let serialized = match fvm_rs_sdk::encoding::to_vec(self) {
                        Ok(s) => s,
                        Err(err) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
                            Some(format!("failed to serialize state: {:?}", err).as_str()),
                        ),
                    };
                    let cid = match fvm_rs_sdk::syscall::ipld::put(
                        fvm_rs_sdk::cid::Code::Blake2b256.into(),
                        fvm_rs_sdk::state::cbor::SIZE,
                        fvm_rs_sdk::encoding::DAG_CBOR,
                        serialized.as_slice(),
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
                    let (name, token_stream) = match &field.ident {
                        Some(ident) => (ident.to_string(), ident.to_token_stream().clone()),
                        _ => unreachable!(),
                    };

                    fields.push(ast::StateStructField {
                        rust_name: token_stream,
                        name,
                        struct_name: s.ident.clone(),
                        ty: field.ty.clone(),
                    });
                }

                let ast_struct = ast::StateStruct {
                    rust_name: s.ident.to_token_stream(),
                    name: s.ident.to_string(),
                    fields,
                    codec: DagCbor,
                };

                // Create ast::Program
                let program = ast::Program {
                    state_structs: vec![ast_struct],
                    actor_implementation: None,
                    payload_structs: vec![],
                };

                program.try_to_tokens(&mut token_stream).unwrap();

                assert_eq!(token_stream.to_string(), expected_final_stream.to_string());
            }
            _ => unreachable!(),
        }
    }
}
