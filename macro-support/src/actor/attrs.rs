use crate::utils::{generate_attr_getters, generate_attrs};
use backend::actor::attrs::{ActorAttr, Dispatch};
use syn::parse::{Parse, ParseStream, Result};

// Parsed attributes from a `#[fvm_actor(..)]`.
generate_attrs!(ActorAttrs, ActorAttr);

// Generate getters to retrieve attributes values
generate_attr_getters!(ActorAttrs, [(dispatch, ActorAttr::Dispatch, Dispatch),]);
