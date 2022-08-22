//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use crate::utils::{ConvertToAst, MacroParse};
use backend::ast::ActorEntryPoint;
use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, ImplItem, Item, Type};

use crate::actor::error::Error::{
    ExpectedImplementation, GenericsOnInterface, UnexpectedImplementationType,
};
use crate::export::attrs::ExportAttrs;

impl<'a> ConvertToAst<()> for &'a mut syn::ItemImpl {
    type Target = ast::ActorImplementation;

    fn convert(self, _: ()) -> Result<Self::Target, Diagnostic> {
        // Not handling generics on actor
        if !self.generics.params.is_empty() {
            return Err(Diagnostic::error(format!("{}", GenericsOnInterface)));
        }

        // Get impl name & ident
        let (rust_name, name) = match self.self_ty.as_ref() {
            Type::Path(type_path) => (
                type_path.path.to_token_stream(),
                type_path.path.to_token_stream().to_string(),
            ),
            _ => {
                return Err(Diagnostic::error(format!(
                    "{}",
                    UnexpectedImplementationType
                )))
            }
        };

        // Initialize entry points
        let mut entry_points: Vec<ActorEntryPoint> = vec![];

        for item in &self.items {
            if let ImplItem::Method(mut m) = item.clone() {
                // Get export token stream
                let filtered_attributes: Vec<&Attribute> = m
                    .attrs
                    .iter()
                    .filter(|a| match a.path.segments.last() {
                        Some(s) => s.to_token_stream().to_string().contains("fvm_export"),
                        None => false,
                    })
                    .collect::<Vec<&Attribute>>();
                if filtered_attributes.is_empty() {
                    continue;
                }

                // Parse export attributes
                let fvm_export_attr: &Attribute = filtered_attributes[0];
                let export_attrs: ExportAttrs = fvm_export_attr.parse_args()?;

                // Generate ast entry point
                let entry_point: ActorEntryPoint = (&mut m).convert(export_attrs)?;
                entry_points.push(entry_point);
            }
        }

        Ok(ast::ActorImplementation {
            rust_name,
            name,
            entry_points,
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
            Item::Impl(mut i) => {
                program.actor_implementation = Some((&mut i).convert(())?);
                i.to_tokens(tokens);
            }
            _ => {
                return Err(Diagnostic::error(format!("{}", ExpectedImplementation)));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use backend::ast::ActorImplementation;
    use quote::quote;

    use super::*;

    #[test]
    fn implementation_to_ast() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                pub fn new() -> Self {
                    Actor {
                        count: 0
                    }
                }
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Parse struct
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, &mut tokens).unwrap();

        let actor_implementation: &ActorImplementation = &program.actor_implementation.unwrap();

        assert_eq!(actor_implementation.name, String::from("Actor"));
    }

    #[test]
    fn implementation_with_generics() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl<T> Actor<T> {
                pub fn value(&self) -> &T {
                    &self.gen_val
                }
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Parse struct
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        if let Err(err) = item.macro_parse(&mut program, &mut tokens) {
            assert_eq!(
                err.to_token_stream().to_string(),
                "compile_error ! { \"implementation with #[fvm_actor] cannot have lifetime or type parameters.\" }"
            )
        } else {
            panic!("implementation with generics and #[fvm_actor] should cause an error")
        }
    }
}
