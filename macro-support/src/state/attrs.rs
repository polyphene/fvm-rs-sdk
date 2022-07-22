use backend::state::attrs::{Codec, StateAttr};
use syn::parse::{Parse, ParseStream, Result};

/// Parsed attributes from a `#[fvm_state(..)]`.
#[derive(Debug, Default)]
pub struct StateAttrs {
    /// List of parsed attributes
    pub attrs: Vec<StateAttr>,
}

impl StateAttrs {
    pub fn codec(&self) -> Option<&Codec> {
        self.attrs
            .iter()
            .map(|a| match &a {
                StateAttr::Codec(codec) => codec,
            })
            .next()
    }
}

impl Parse for StateAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = StateAttrs::default();
        if input.is_empty() {
            return Ok(attrs);
        }

        let opts =
            syn::punctuated::Punctuated::<StateAttr, syn::token::Comma>::parse_terminated(input)?;
        attrs.attrs = opts.into_iter().collect();

        Ok(attrs)
    }
}