//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use backend::state::attrs::Codec;
use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ImplItem, Item};

use crate::actor::attrs::ActorAttrs;

/// Conversion trait with context.
///
/// Used to convert syn tokens into an AST, that we can then use to generate glue code.
trait ConvertToAst<Ctx> {
    /// What we are converting to.
    type Target;
    /// Convert into our target.
    ///
    /// Since this is used in a procedural macro, use panic to fail.
    fn convert(self, context: Ctx) -> Result<Self::Target, Diagnostic>;
}

impl<'a> ConvertToAst<ActorAttrs> for &'a mut syn::ItemImpl {
    type Target = ast::ActorImplementation;

    fn convert(self, _attrs: ActorAttrs) -> Result<Self::Target, Diagnostic> {
        // Generate the AST object for the Struct
        for item in &self.items {
            match item {
                ImplItem::Const(c) => {
                    let mut const_stream = TokenStream::new();
                    c.to_tokens(&mut const_stream);
                    dbg!(const_stream.to_string());
                }
                ImplItem::Method(m) => {
                    let mut method_stream = TokenStream::new();
                    m.to_tokens(&mut method_stream);
                    dbg!(method_stream.to_string());
                }
                ImplItem::Type(t) => {
                    let mut type_stream = TokenStream::new();
                    t.to_tokens(&mut type_stream);
                    dbg!(type_stream.to_string());
                }
                ImplItem::Macro(m) => {
                    let mut macro_stream = TokenStream::new();
                    m.to_tokens(&mut macro_stream);
                    dbg!(macro_stream.to_string());
                }
                ImplItem::Verbatim(v) => {
                    let mut verba_stream = TokenStream::new();
                    v.to_tokens(&mut verba_stream);
                    dbg!(verba_stream.to_string());
                }
                _ => unreachable!(),
            }
        }

        Ok(ast::ActorImplementation {
            name: String::from("hello"),
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
                //dbg!(&tokens.to_string());
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
    use backend::state::attrs::Codec;
    use quote::quote;
    use syn::{Member, Type};

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
        let attrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        assert!(true)
    }
}
