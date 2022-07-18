//! Contains all structures that can be parsed from a `TokenStream`. They will be used when generating
//! code

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn;

use crate::state::attrs::Codec;

/// An abstract syntax tree representing a rust program.
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Default, Clone)]
pub struct Program {
    /// state rust structs
    pub state_structs: Vec<StateStruct>,
}

/// Information about a Struct being used as state object
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct StateStruct {
    /// The name of the struct in Rust code
    pub rust_name: Ident,
    /// The name of the struct for the SDK
    pub name: String,
    /// All the fields of this struct to export
    pub fields: Vec<StateStructField>,
    /// Codec used to store state
    pub codec: Codec,
}

impl StateStruct {
    pub fn generate_state_interface(&self) -> TokenStream {
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

                            // Get state's bytes
                            match fvm_rs_sdk::state::cbor::CborBlockstore.get_cbor(&root) {
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
                            use fvm_rs_sdk::encoding::CborStore;

                            match fvm_rs_sdk::state::cbor::CborBlockstore
                                .put_cbor(self, fvm_rs_sdk::cid::Code::Blake2b256.into())
                            {
                                Ok(cid) => cid,
                                Err(err) => fvm_rs_sdk::syscall::vm::abort(
                                    fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
                                    Some(format!("failed to store state: {:}", err).as_str()),
                                ),
                            }

                            if let Err(err) = fvm_rs_sdk::syscall::sself::set_root(&cid) {
                                fvm_rs_sdk::syscall::vm::abort(
                                    fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                                    Some(format!("failed to set root cid: {:}", err).as_str()),
                                );
                            }
                            cid
                        }
                    }
                )
            }
        }
    }
}

/// The field of a struct
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct StateStructField {
    /// The name of the field in Rust code
    pub rust_name: syn::Member,
    /// The name of the field in code
    pub name: String,
    /// The name of the struct this field is part of
    pub struct_name: Ident,
    /// The type of this field
    pub ty: syn::Type,
}
