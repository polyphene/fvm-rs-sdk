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
        // No lifetime to make sure that we can handle it correctly
        if self.generics.params.len() > 0 {
            bail_span!(
                self.generics,
                "structs with #[fvm_state] cannot have lifetime or type parameters currently"
            );
        }

        // When handling struct, first create fields objects
        let mut fields = Vec::new();
        for (i, field) in self.fields.iter_mut().enumerate() {
            // Fields visibility has to be public to be taken into account
            match field.vis {
                syn::Visibility::Public(..) => {}
                _ => continue,
            }

            // Derive field name from ident
            let (name, member) = match &field.ident {
                Some(ident) => (ident.to_string(), syn::Member::Named(ident.clone())),
                None => (i.to_string(), syn::Member::Unnamed(i.into())),
            };

            fields.push(ast::StructField {
                rust_name: member,
                name,
                struct_name: self.ident.clone(),
                ty: field.ty.clone(),
            });
        }

        // Generate the AST object for the Struct
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
            // Handles strcutures
            syn::Item::Struct(mut s) => {
                program.structs.push((&mut s).convert()?);
                s.to_tokens(tokens);
            }
            _ => {
                bail_span!(self, "#[fvm_state] can only be applied to a public struct",);
            }
        }

        Ok(())
    }
}
