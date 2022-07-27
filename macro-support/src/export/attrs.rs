use crate::utils::{generate_attr_getters, generate_attrs};
use backend::export::attrs::{Binding, ExportAttr};
use syn::parse::{Parse, ParseStream, Result};

#[derive(Debug, Default)]
pub(crate) struct ExportAttrs {
    pub attrs: Vec<ExportAttr>,
}

impl Parse for ExportAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = ExportAttrs::default();
        if input.is_empty() {
            return Ok(attrs);
        }

        let opts =
            syn::punctuated::Punctuated::<ExportAttr, syn::token::Comma>::parse_terminated(input)?;
        attrs.attrs = opts.into_iter().collect();

        Ok(attrs)
    }
}

generate_attr_getters!(ExportAttrs, [(binding, ExportAttr::Binding, Binding),]);
