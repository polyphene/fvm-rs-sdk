use crate::utils::{generate_attr_getters, generate_attrs};
use backend::state::attrs::{Codec, StateAttr};
use syn::parse::{Parse, ParseStream, Result};

// Parsed attributes from a `#[fvm_state(..)]`.
generate_attrs!(StateAttrs, StateAttr);

// Generate getters to retrieve attributes values
generate_attr_getters!(StateAttrs, [(codec, StateAttr::Codec, Codec),]);
