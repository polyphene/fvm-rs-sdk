//! Contains attributes available for the `#[fvm_actor]` procedural macro.

use std::convert::{TryFrom, TryInto};

use crate::utils::AnyIdent;
use anyhow::Result;
use syn::parse::{Parse, ParseStream, Result as SynResult};

use crate::export::error::Error::{InvalidBindingValue, InvalidNumericValue, UnknownAttribute};

#[derive(Clone, Debug)]
pub enum ExportAttr {
    BindingMethod(Method),
}

impl TryFrom<String> for ExportAttr {
    type Error = crate::export::error::Error;

    fn try_from(attr: String) -> Result<Self, Self::Error> {
        match attr.as_str() {
            "method_num" => Ok(ExportAttr::BindingMethod(Method::default())),
            _ => Err(UnknownAttribute(attr)),
        }
    }
}

impl Parse for ExportAttr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let original = input.fork();
        let attr: AnyIdent = input.parse()?;
        let attr = attr.0;

        match ExportAttr::try_from(attr.to_string()) {
            Ok(ExportAttr::BindingMethod(Method::Numeric(_))) => {
                input.parse::<syn::token::Eq>()?;
                // Try to get value from parsing an integer
                if let Ok(num) = input.parse::<syn::LitInt>() {
                    return Ok(ExportAttr::BindingMethod(Method::Numeric(
                        num.base10_parse::<u64>().map_err(|_| {
                            original.error(format!("{}", InvalidNumericValue(num.to_string())))
                        })?,
                    )));
                }
                Err(original.error(format!("{}", InvalidBindingValue)))
            }
            Err(err) => Err(original.error(format!("{}", err))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Method {
    Numeric(u64),
}

impl TryInto<u64> for Method {
    type Error = crate::export::error::Error;

    fn try_into(self) -> std::result::Result<u64, Self::Error> {
        match self {
            Method::Numeric(num) => Ok(num),
        }
    }
}

impl Default for Method {
    fn default() -> Self {
        Method::Numeric(0)
    }
}
