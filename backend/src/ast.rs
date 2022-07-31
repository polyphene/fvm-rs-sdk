//! Contains all structures that can be parsed from a `TokenStream`. They will be used when generating
//! code

use crate::actor::attrs::Dispatch;
use crate::export::attrs::Binding;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn;

use crate::state::attrs::Codec;
use crate::{Diagnostic, TryToTokens};

/// An abstract syntax tree representing a rust program.
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Default, Clone)]
pub struct Program {
    /// state rust structs
    pub state_structs: Vec<StateStruct>,
    /// Actor implementation
    pub actor_implementation: Option<ActorImplementation>,
}

impl TryToTokens for Program {
    // Generate wrappers for all the items that we've found
    fn try_to_tokens(&self, into: &mut TokenStream) -> Result<(), Diagnostic> {
        // Handling tagged structures
        for s in self.state_structs.iter() {
            s.to_tokens(into);
        }

        if let Some(actor_implementation) = &self.actor_implementation {
            actor_implementation.to_tokens(into);
        }

        Ok(())
    }
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
    /// The name of the implementation in Rust code
    pub rust_name: syn::Member,
    /// The name of the implementation in code
    pub name: String,
    /// The internal dispatch method selected for the actor
    pub dispatch: Dispatch,
    /// The entry points that are available for the actor
    pub entry_points: Vec<ActorEntryPoint>,
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
    /// The mutability of the method
    pub mutability: Mutability,
    /// Boolean to know if entry point return data
    pub returns: bool,
    /// Arguments expected by the method
    pub arguments: Vec<MethodArgument>,
}

/// Information about the mutability of an entry point
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub enum Mutability {
    // No read or write on state
    Pure,
    // Read on state
    View,
    // Read and write on state
    Write,
}

/// Information about an argument for a method used as an entry point
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct MethodArgument {
    /// Boolean to know if the parameter should be mutable
    pub mutable: bool,
    /// The internal entry point value specified for the method
    pub arg_type: syn::Member,
}
