//! Contains attributes available for the `#[fvm_actor]` procedural macro.

use std::convert::TryFrom;

use crate::utils::AnyIdent;
use anyhow::Result;
use syn::parse::{Parse, ParseStream, Result as SynResult};

use crate::export::error::Error::{InvalidBindingValue, InvalidNumericValue, UnknownAttribute};

#[derive(Clone, Debug)]
pub enum ExportAttr {
    Binding(Binding),
}

impl TryFrom<String> for ExportAttr {
    type Error = crate::export::error::Error;

    fn try_from(attr: String) -> Result<Self, Self::Error> {
        match attr.as_str() {
            "binding" => Ok(ExportAttr::Binding(Binding::default())),
            _ => Err(UnknownAttribute(attr)),
        }
    }
}

impl Parse for ExportAttr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let original = input.fork();
        let attr: AnyIdent = input.parse()?;
        let attr = attr.0;

        return match ExportAttr::try_from(attr.to_string()) {
            Ok(ExportAttr::Binding(_)) => {
                input.parse::<syn::token::Eq>()?;
                // Try to get value from parsing an integer
                match input.parse::<syn::LitInt>() {
                    Ok(num) => {
                        return Ok(ExportAttr::Binding(Binding::Numeric(
                            num.base10_parse::<u64>()
                                .or(Err(original
                                    .error(format!("{}", InvalidNumericValue(num.to_string())))))?,
                        )));
                    }
                    _ => {}
                };
                return Err(original.error(format!("{}", InvalidBindingValue)));
            }
            Err(err) => Err(original.error(format!("{}", err))),
        };
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    Numeric(u64),
}

impl Default for Binding {
    fn default() -> Self {
        Binding::Numeric(0)
    }
}
