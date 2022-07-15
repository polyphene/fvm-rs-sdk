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
                quote!()
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
