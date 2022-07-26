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
    use quote::quote;

    use super::*;

    #[test]
    fn mock_actor() {
        // Mock struct token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export(method-num="1")]
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

        assert!(true)
    }
}
