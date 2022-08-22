use crate::utils::{generate_attr_getters, generate_attrs};
use backend::payload::attrs::{Codec, PayloadAttr};
use syn::parse::{Parse, ParseStream, Result};

// Parsed attributes from a `#[fvm_payload(..)]`.
generate_attrs!(PayloadAttrs, PayloadAttr);

// Generate getters to retrieve attributes values
generate_attr_getters!(PayloadAttrs, [(codec, PayloadAttr::Codec, Codec),]);
