//! Contains attributes available for the `#[fvm_actor]` procedural macro.

use std::convert::TryFrom;

use crate::utils::AnyIdent;
use anyhow::Result;
use syn::parse::{Parse, ParseStream, Result as SynResult};

use crate::actor::error::Error::{
    InvalidDispatchMethodFormat, UnknownAttribute, UnkownDispatchMethod,
};

#[derive(Clone, Debug)]
pub enum ActorAttr {
    Dispatch(Dispatch),
}

impl TryFrom<String> for ActorAttr {
    type Error = crate::actor::error::Error;

    fn try_from(attr: String) -> Result<Self, Self::Error> {
        match attr.as_str() {
            "dispatch" => Ok(ActorAttr::Dispatch(Dispatch::default())),
            _ => Err(UnknownAttribute(attr)),
        }
    }
}

impl Parse for ActorAttr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let original = input.fork();
        let attr: AnyIdent = input.parse()?;
        let attr = attr.0;

        match ActorAttr::try_from(attr.to_string()) {
            Ok(ActorAttr::Dispatch(_)) => {
                input.parse::<syn::token::Eq>()?;
                let val = match input.parse::<syn::LitStr>() {
                    Ok(str) => match Dispatch::try_from(str.value()) {
                        Ok(state_attr) => state_attr,
                        Err(err) => return Err(original.error(format!("{}", err))),
                    },
                    Err(err) => {
                        return Err(original
                            .error(format!("{}", InvalidDispatchMethodFormat(err.to_string()))))
                    }
                };
                Ok(ActorAttr::Dispatch(val))
            }
            Err(err) => Err(original.error(format!("{}", err))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dispatch {
    Numeric,
}

impl Default for Dispatch {
    fn default() -> Self {
        Dispatch::Numeric
    }
}

impl TryFrom<String> for Dispatch {
    type Error = crate::actor::error::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "method-num" => Ok(Dispatch::Numeric),
            _ => Err(UnkownDispatchMethod(value)),
        }
    }
}
