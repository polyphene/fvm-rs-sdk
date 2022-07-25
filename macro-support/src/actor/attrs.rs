use backend::state::attrs::{Codec, StateAttr};
use syn::parse::{Parse, ParseStream, Result};

/// Parsed attributes from a `#[fvm_actor(..)]`.
#[derive(Debug, Default)]
pub struct ActorAttrs {}

impl Parse for ActorAttrs {
    fn parse(_input: ParseStream) -> Result<Self> {
        let mut attrs = ActorAttrs::default();

        Ok(attrs)
    }
}
