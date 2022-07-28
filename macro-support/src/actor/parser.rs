//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use crate::utils::{ConvertToAst, MacroParse};
use backend::actor::attrs::Dispatch;
use backend::ast::ActorEntryPoint;
use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, ImplItem, Item, Type};

use crate::actor::attrs::ActorAttrs;
use crate::actor::error::Error::{
    ExpectedImplementation, GenericsOnInterface, UnexpectedImplementationType,
};
use crate::export::attrs::ExportAttrs;

impl<'a> ConvertToAst<ActorAttrs> for &'a mut syn::ItemImpl {
    type Target = ast::ActorImplementation;

    fn convert(self, attrs: ActorAttrs) -> Result<Self::Target, Diagnostic> {
        // Not handling generics on actor
        if !self.generics.params.is_empty() {
            return Err(Diagnostic::error(format!("{}", GenericsOnInterface)));
        }

        // Attrs assignment
        let dispatch = match attrs.dispatch() {
            Some(dispatch) => dispatch.clone(),
            None => Dispatch::default(),
        };

        // Get impl name & ident
        let (rust_name, name) = match self.self_ty.as_ref() {
            Type::Path(type_path) => {
                let path_ident = match type_path.path.get_ident() {
                    Some(ident) => ident.clone(),
                    None => {
                        return Err(Diagnostic::error(format!(
                            "{}",
                            UnexpectedImplementationType
                        )))
                    }
                };
                (
                    syn::Member::Named(path_ident.clone()),
                    path_ident.to_string(),
                )
            }
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
                    .filter(|a| {
                        a.path
                            .get_ident()
                            .unwrap()
                            .to_string()
                            .contains("fvm_export")
                    })
                    .collect::<Vec<&Attribute>>();
                if filtered_attributes.is_empty() {
                    continue;
                }

                // Parse export attributes
                let fvm_export_attr: &Attribute = filtered_attributes[0];
                let export_attrs: ExportAttrs = fvm_export_attr.parse_args()?;

                // Generate ast entry point
                let entry_point: ActorEntryPoint = (&mut m).convert((&dispatch, export_attrs))?;
                entry_points.push(entry_point);
            }
        }

        Ok(ast::ActorImplementation {
            rust_name,
            name,
            dispatch,
            entry_points,
        })
    }
}

impl<'a> MacroParse<(Option<ActorAttrs>, &'a mut TokenStream)> for syn::Item {
    fn macro_parse(
        self,
        program: &mut ast::Program,
        (attrs, tokens): (Option<ActorAttrs>, &'a mut TokenStream),
    ) -> Result<(), Diagnostic> {
        // Match of Item types to parse & generate our AST
        match self {
            Item::Impl(mut i) => {
                let attrs = attrs.unwrap_or_default();
                program.actor_implementation = Some((&mut i).convert(attrs)?);
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

        // Mock attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {
            dispatch = "method-num"
        })
        .to_tokens(&mut attrs_token_stream);

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: ActorAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let actor_implementation: &ActorImplementation = &program.actor_implementation.unwrap();

        assert_eq!(actor_implementation.dispatch, Dispatch::Numeric);
        assert_eq!(actor_implementation.name, String::from("Actor"));
    }

    #[test]
    fn implementation_no_attrs_to_ast() {
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

        // Mock no attrs
        let attrs_token_stream = TokenStream::new();

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: ActorAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let actor_implementation: &ActorImplementation = &program.actor_implementation.unwrap();

        assert_eq!(actor_implementation.dispatch, Dispatch::Numeric);
    }

    #[test]
    fn impl_with_unknown_attr() {
        // Mock no attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {
            john = "method-num"
        })
        .to_tokens(&mut attrs_token_stream);

        // Parse impl and attrs
        match syn::parse2::<ActorAttrs>(attrs_token_stream) {
            Err(err) => assert_eq!(err.to_string(), "unknown attribute 'john'"),
            _ => panic!("unknown attribute should throw an error"),
        }
    }

    #[test]
    fn impl_with_unknown_codec() {
        // Mock no attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {
            dispatch = "john"
        })
        .to_tokens(&mut attrs_token_stream);

        // Parse impl and attrs
        match syn::parse2::<ActorAttrs>(attrs_token_stream) {
            Err(err) => assert_eq!(err.to_string(), "unknown dispatch method 'john'"),
            _ => panic!("unknown attribute should throw an error"),
        }
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

        // Mock no attrs
        let attrs_token_stream = TokenStream::new();

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: ActorAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        if let Err(err) = item.macro_parse(&mut program, (Some(attrs), &mut tokens)) {
            assert_eq!(
                err.to_token_stream().to_string(),
                "compile_error ! { \"implementation with #[fvm_actor] cannot have lifetime or type parameters.\" }"
            )
        } else {
            panic!("implementation with generics and #[fvm_actor] should cause an error")
        }
    }

    #[test]
    fn implementation_with_unexpected_ident() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl crate::state::Actor {
                pub fn new() -> Self {
                    Actor {
                        count: 0
                    }
                }
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Mock attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {
            dispatch = "method-num"
        })
        .to_tokens(&mut attrs_token_stream);

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: ActorAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        if let Err(err) = item.macro_parse(&mut program, (Some(attrs), &mut tokens)) {
            assert_eq!(
                err.to_token_stream().to_string(),
                "compile_error ! { \"expected implementation for type with no leading colon, 1 path segment, and no angle bracketed or parenthesized path arguments with #[fvm_actor]\" }"
            )
        } else {
            panic!("implementation with improper path identity should cause an error")
        }
    }
}
