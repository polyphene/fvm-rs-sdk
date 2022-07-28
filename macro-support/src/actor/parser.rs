//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use crate::utils::{ConvertToAst, MacroParse};
use backend::actor::attrs::Dispatch;
use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ImplItem, Item};

use crate::actor::attrs::ActorAttrs;

impl<'a> ConvertToAst<ActorAttrs> for &'a mut syn::ItemImpl {
    type Target = ast::ActorImplementation;

    fn convert(self, attrs: ActorAttrs) -> Result<Self::Target, Diagnostic> {
        // Generate the AST object for the Struct
        for item in &self.items {
            if let ImplItem::Method(_m) = item {
                // TODO Should parse #[fvm_export] here
            }
        }

        // Attrs assignment
        let dispatch = match attrs.dispatch() {
            Some(dispatch) => dispatch.clone(),
            None => Dispatch::default(),
        };

        Ok(ast::ActorImplementation { dispatch })
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
                program.actor_implementation.push((&mut i).convert(attrs)?);
                i.to_tokens(tokens);
            }
            _ => {
                bail_span!(
                    self,
                    "#[fvm_actor] can only be applied to an implementation",
                );
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

        let actor_implementation: &ActorImplementation = &program.actor_implementation[0];

        assert_eq!(actor_implementation.dispatch, Dispatch::Numeric);
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

        let actor_implementation: &ActorImplementation = &program.actor_implementation[0];

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
}
