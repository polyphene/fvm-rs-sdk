//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn;

/// Conversion trait with context.
///
/// Used to convert syn tokens into an AST, that we can then use to generate glue code.
trait ConvertToAst {
    /// What we are converting to.
    type Target;
    /// Convert into our target.
    ///
    /// Since this is used in a procedural macro, use panic to fail.
    fn convert(self) -> Result<Self::Target, Diagnostic>;
}

impl<'a> ConvertToAst for &'a mut syn::ItemStruct {
    type Target = ast::Struct;

    fn convert(self) -> Result<Self::Target, Diagnostic> {
        // TODO Implement

        // When handling struct, first create fields objects
        let mut fields = Vec::new();

        Ok(ast::Struct {
            rust_name: self.ident.clone(),
            name: self.ident.to_string(),
            fields,
        })
    }
}

pub(crate) trait MacroParse<Ctx> {
    /// Parse the contents of an object into our AST, with a context if necessary.
    ///
    /// The context is used to have access to the attributes on `#[fvm_state]`, and to allow
    /// writing to the output `TokenStream`.
    fn macro_parse(self, program: &mut ast::Program, context: Ctx) -> Result<(), Diagnostic>;
}

impl<'a> MacroParse<&'a mut TokenStream> for syn::Item {
    fn macro_parse(
        self,
        program: &mut ast::Program,
        tokens: &'a mut TokenStream,
    ) -> Result<(), Diagnostic> {
        // Match of Item types to parse & generate our AST
        match self {
            _ => {
                bail_span!(self, "TODO Implement for struct",);
            }
        }

        Ok(())
    }
}
