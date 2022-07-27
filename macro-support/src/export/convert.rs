//! Convert reads a source `TokenStream` to prepare the backend to generate custom code

use crate::utils::ConvertToAst;
use backend::actor::attrs::Dispatch;
use backend::export::attrs::Binding;
use backend::{ast, Diagnostic};

use crate::export::attrs::ExportAttrs;
use crate::export::error::Error::{MismatchedDispatchBinding, MissingBinding};

impl<'a> ConvertToAst<(&Dispatch, ExportAttrs)> for &'a mut syn::ImplItemMethod {
    type Target = ast::ActorEntryPoint;

    fn convert(
        self,
        (dispatch, attrs): (&Dispatch, ExportAttrs),
    ) -> Result<Self::Target, Diagnostic> {
        // Get binding value
        let binding_value: &Binding = attrs.binding().ok_or_else(|| {
            Diagnostic::error(format!("{}", MissingBinding(self.sig.ident.to_string())))
        })?;
        // Get dispatch method
        match dispatch {
            Dispatch::Numeric => {
                // Match binding value
                match binding_value {
                    // For numeric dispatch we expect an integer value
                    Binding::Numeric(value) => Ok(ast::ActorEntryPoint {
                        rust_name: syn::Member::Named(self.sig.ident.clone()),
                        name: self.sig.ident.to_string(),
                        binding: Binding::Numeric(*value),
                    }),
                    #[allow(unreachable_patterns)]
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
    use backend::export::attrs::Binding;
    use proc_macro2::TokenStream;
    use quote::quote;
    use quote::ToTokens;

    #[test]
    fn export_to_ast() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export(binding=1)]
                pub fn new() -> Self {
                    Actor {
                        count: 0
                    }
                }

                #[fvm_export(binding=2)]
                pub fn from(value: u64) -> Self {
                    Actor {
                        count: value
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

        let actor_entry_points = &program.actor_implementation[0].entry_points;
        assert_eq!(actor_entry_points.len(), 2);

        assert_eq!(actor_entry_points[0].name, String::from("new"));
        assert_eq!(actor_entry_points[0].binding, Binding::Numeric(1));

        assert_eq!(actor_entry_points[1].name, String::from("from"));
        assert_eq!(actor_entry_points[1].binding, Binding::Numeric(2));
    }
    #[test]
    fn no_binding() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export]
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
            "compile_error ! { \"expected attribute arguments in parentheses: #[fvm_export(...)]\" }"
        )
        }
    }

    #[test]
    fn bad_binding_type() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export(binding="toto")]
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
                "compile_error ! { \"invalid binding value\" }"
            )
        }
    }

    #[test]
    fn unknown_attribute() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export(hello=1)]
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
                "compile_error ! { \"unknown attribute 'hello'\" }"
            )
        }
    }
}
