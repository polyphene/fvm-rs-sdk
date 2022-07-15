//! Contains attributes available for the `#[fvm_state]` procedural macro.

use std::convert::TryFrom;

use anyhow::Result;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream, Result as SynResult};

use crate::state::error::Error::{UnknownAttribute, UnknownCodec};

// AnyIdent is a wrapper around Ident to be able to implement Parse trait
struct AnyIdent(Ident);

impl Parse for AnyIdent {
    fn parse(input: ParseStream) -> SynResult<Self> {
        input.step(|cursor| match cursor.ident() {
            Some((ident, remaining)) => Ok((AnyIdent(ident), remaining)),
            None => Err(cursor.error("expected an identifier")),
        })
    }
}

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
                    Err(_) => {
                        let ident = input.parse::<AnyIdent>()?.0;
                        dbg!(ident.to_string());
                        panic!("aa")
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
