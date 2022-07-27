//! Contains attributes available for the `#[fvm_state]` procedural macro.

use std::convert::TryFrom;

use crate::utils::AnyIdent;
use anyhow::Result;
use syn::parse::{Parse, ParseStream, Result as SynResult};

use crate::state::error::Error::{InvalidCodecFormat, UnknownAttribute, UnknownCodec};

#[derive(Clone, Debug)]
pub enum StateAttr {
    Codec(Codec),
}

impl TryFrom<String> for StateAttr {
    type Error = crate::state::error::Error;

    fn try_from(attr: String) -> Result<Self, Self::Error> {
        match attr.as_str() {
            "codec" => Ok(StateAttr::Codec(Codec::default())),
            _ => Err(UnknownAttribute(attr)),
        }
    }
}

impl Parse for StateAttr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let original = input.fork();
        let attr: AnyIdent = input.parse()?;
        let attr = attr.0;

        return match StateAttr::try_from(attr.to_string()) {
            Ok(StateAttr::Codec(_)) => {
                input.parse::<syn::token::Eq>()?;
                let val = match input.parse::<syn::LitStr>() {
                    Ok(str) => match Codec::try_from(str.value()) {
                        Ok(state_attr) => state_attr,
                        Err(err) => return Err(original.error(format!("{}", err))),
                    },
                    Err(err) => {
                        return Err(
                            original.error(format!("{}", InvalidCodecFormat(err.to_string())))
                        )
                    }
                };
                Ok(StateAttr::Codec(val))
            }
            Err(err) => Err(original.error(format!("{}", err))),
        };
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Codec {
    DagCbor,
}

impl Default for Codec {
    fn default() -> Self {
        Codec::DagCbor
    }
}

impl TryFrom<String> for Codec {
    type Error = crate::state::error::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "dag-cbor" => Ok(Codec::DagCbor),
            _ => Err(UnknownCodec(value)),
        }
    }
}
