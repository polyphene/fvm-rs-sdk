//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;

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
        if !self.generics.params.is_empty() {
            bail_span!(
                self.generics,
                "structs with #[fvm_state] cannot have lifetime or type generic parameters"
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

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{Member, Type};

    use super::*;

    #[test]
    fn struct_to_ast() {
        let mut token_stream = TokenStream::new();

        (quote! {
            pub struct MockStruct {
                pub count: u64
            }
        })
        .to_tokens(&mut token_stream);
        let item = syn::parse2::<syn::Item>(token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, &mut tokens).unwrap();

        let parsed_struct = &program.structs[0];
        let parsed_field = &parsed_struct.fields[0];

        assert_eq!(parsed_struct.name, "MockStruct");
        assert_eq!(parsed_struct.rust_name.to_string(), parsed_struct.name);

        assert_eq!(parsed_field.name, "count");
        assert_eq!(parsed_field.struct_name.to_string(), parsed_struct.name);
        match &parsed_field.ty {
            Type::Path(type_path) => {
                assert_eq!(
                    type_path.path.segments.last().unwrap().ident.to_string(),
                    "u64"
                );
            }
            _ => {
                panic!("count type should be path")
            }
        }
        match &parsed_field.rust_name {
            Member::Named(ident) => {
                assert_eq!(ident.to_string(), parsed_field.name);
            }
            _ => panic!("parsed struct field rust name should be named"),
        }
    }

    #[test]
    fn no_struct_with_lifetime() {
        let mut token_stream = TokenStream::new();

        (quote! {
            struct MockStruct<'a> {
                pub count: &'a u64,
            }
        })
        .to_tokens(&mut token_stream);
        let item = syn::parse2::<syn::Item>(token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        let res_parse = item.macro_parse(&mut program, &mut tokens);

        match res_parse {
            Err(diagnostic) => {
                let res_panic = std::panic::catch_unwind(|| diagnostic.panic());
                match res_panic {
                    Err(err) => match err.downcast::<String>() {
                        Ok(panic_msg_box) => {
                            assert_eq!(panic_msg_box.as_str(), "structs with #[fvm_state] cannot have lifetime or type generic parameters");
                        }
                        Err(_) => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => panic!("parse result should be error when struct has lifetime"),
        }
    }

    #[test]
    fn private_fields_not_parsed() {
        let mut token_stream = TokenStream::new();

        (quote! {
            pub struct MockStruct {
                pub count: u64,
                count_bis: u64
            }
        })
        .to_tokens(&mut token_stream);
        let item = syn::parse2::<syn::Item>(token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, &mut tokens).unwrap();

        let parsed_struct = &program.structs[0];
        let parsed_field = &parsed_struct.fields[0];

        assert_eq!(parsed_struct.fields.len(), 1usize);
        assert_eq!(parsed_field.name, "count");
    }
}
