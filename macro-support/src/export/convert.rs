//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use crate::utils::{ConvertToAst, MacroParse};
use backend::actor::attrs::Dispatch;
use backend::export::attrs::Binding;
use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ImplItem, Item};

use crate::export::attrs::ExportAttrs;
use crate::export::error::Error::{MismatchedDispatchBinding, MissingBinding};

impl<'a> ConvertToAst<(&Dispatch, ExportAttrs)> for &'a mut syn::ImplItemMethod {
    type Target = ast::ActorEntryPoint;

    fn convert(
        self,
        (dispatch, attrs): (&Dispatch, ExportAttrs),
    ) -> Result<Self::Target, Diagnostic> {
        // Get binding value
        let binding_value: &Binding = attrs.binding().ok_or(Diagnostic::error(format!(
            "{}",
            MissingBinding(self.sig.ident.to_string())
        )))?;
        // Get dispatch method
        match dispatch {
            Dispatch::Numeric => {
                // Match binding value
                match binding_value {
                    // For numeric dispatch we expect an integer value
                    Binding::Numeric(value) => Ok(ast::ActorEntryPoint {
                        rust_name: syn::Member::Named(self.sig.ident.clone()),
                        name: self.sig.ident.to_string(),
                        binding: Binding::default(),
                    }),
                    _ => {
                        return Err(Diagnostic::error(format!(
                            "{}",
                            MismatchedDispatchBinding(
                                self.sig.ident.to_string(),
                                String::from("u64")
                            )
                        )))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actor::attrs::ActorAttrs;
    use crate::utils::MacroParse;
    use fvm_rs_sdk_backend::ast::ActorImplementation;
    use proc_macro2::TokenStream;
    use quote::quote;
    use quote::ToTokens;

    #[test]
    fn implementation_to_ast() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[test_macro]
                #[fvm_export(binding=1)]
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

        assert!(false)
    }
}
