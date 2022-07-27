//! Contains all structures that can be parsed from a `TokenStream`. They will be used when generating
//! code

use crate::actor::attrs::Dispatch;
use crate::export::attrs::Binding;
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
    /// Actor implementation
    pub actor_implementation: Vec<ActorImplementation>,
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
                                32,
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

/// Information about an Implementation being used as an actor interface
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct ActorImplementation {
    /// The internal dispatch method selected for the actor
    pub dispatch: Dispatch,
}

/// Information about an entry point being used in an actor
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct ActorEntryPoint {
    /// The name of the method in Rust code
    pub rust_name: syn::Member,
    /// The name of the method in code
    pub name: String,
    /// The internal entry point value specified for the method
    pub binding: Binding,
}
