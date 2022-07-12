//! Contains all structures that can be parsed from a `TokenStream`. They will be used when generating
//! code

use proc_macro2::{Ident, Span};
use syn;

/// An abstract syntax tree representing a rust program.
#[cfg_attr(feature = "extra-traits", derive(Debug))]
#[derive(Default, Clone)]
pub struct Program {
    /// rust structs
    pub structs: Vec<Struct>,
}

/// Information about a Struct being exported
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct Struct {
    /// The name of the struct in Rust code
    pub rust_name: Ident,
    /// The name of the struct for the SDK
    pub name: String,
    /// All the fields of this struct to export
    pub fields: Vec<StructField>,
}

/// The field of a struct
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct StructField {
    /// The name of the field in Rust code
    pub rust_name: syn::Member,
    /// The name of the field in code
    pub name: String,
    /// The name of the struct this field is part of
    pub struct_name: Ident,
    /// The type of this field
    pub ty: syn::Type,
}
