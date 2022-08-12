//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::payload::error::Error::{ExpectedStructure, GenericsOnStructure};
use crate::utils::{ConvertToAst, MacroParse};

impl<'a> ConvertToAst<()> for &'a mut syn::ItemStruct {
    type Target = ast::PayloadStruct;

    fn convert(self, _: ()) -> Result<Self::Target, Diagnostic> {
        // No lifetime to make sure that we can handle it correctly
        if !self.generics.params.is_empty() {
            return Err(Diagnostic::error(format!("{}", GenericsOnStructure)));
        }

        // Generate the AST object for the Struct
        Ok(ast::PayloadStruct {
            rust_name: self.ident.to_token_stream(),
            name: self.ident.to_string(),
        })
    }
}

impl<'a> MacroParse<&'a mut TokenStream> for syn::Item {
    fn macro_parse(
        self,
        program: &mut ast::Program,
        tokens: &'a mut TokenStream,
    ) -> Result<(), Diagnostic> {
        // Match of Item types to parse & generate our AST
        match self {
            // Handles structures
            syn::Item::Struct(mut s) => {
                program.payload_structs.push((&mut s).convert(())?);
                s.to_tokens(tokens);
            }
            _ => {
                return Err(Diagnostic::error(format!("{}", ExpectedStructure)));
            }
        }

        Ok(())
    }
}
