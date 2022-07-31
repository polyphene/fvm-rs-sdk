//! Convert reads a source `TokenStream` to prepare the backend to generate custom code

use crate::utils::ConvertToAst;
use backend::actor::attrs::Dispatch;
use backend::ast::Mutability;
use backend::export::attrs::Binding;
use backend::{ast, Diagnostic};
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{FnArg, GenericArgument, Pat, PathArguments, ReturnType, Type, Visibility};

use crate::export::attrs::ExportAttrs;
use crate::export::error::Error::{
    ExpectedBindingToNewVariable, GenericsOnEntryPoint, MismatchedDispatchBinding, MissingBinding,
    UnexpectedArgReceiver, UnexpectedArgType, UnhandledType, VisbilityNotPublic,
};

impl<'a> ConvertToAst<(&Dispatch, ExportAttrs)> for &'a mut syn::ImplItemMethod {
    type Target = ast::ActorEntryPoint;

    fn convert(
        self,
        (dispatch, attrs): (&Dispatch, ExportAttrs),
    ) -> Result<Self::Target, Diagnostic> {
        // Not handling generics on entry point
        if !self.sig.generics.params.is_empty() {
            return Err(Diagnostic::error(format!(
                "{}",
                GenericsOnEntryPoint(self.sig.ident.to_string())
            )));
        }
        // Visibility should be public
        match &self.vis {
            Visibility::Public(_) => {}
            _ => {
                return Err(Diagnostic::error(format!(
                    "{}",
                    VisbilityNotPublic(self.sig.ident.to_string())
                )))
            }
        }

        // Get binding value
        let binding_value: &Binding = attrs.binding().ok_or_else(|| {
            Diagnostic::error(format!("{}", MissingBinding(self.sig.ident.to_string())))
        })?;
        // Get mutability for method
        let mutability = match self.sig.inputs.first() {
            Some(arg) => match arg {
                FnArg::Receiver(receiver) => {
                    if receiver.mutability.is_some() {
                        Mutability::Write
                    } else {
                        Mutability::View
                    }
                }
                FnArg::Typed(_) => Mutability::Pure,
            },
            None => Mutability::Pure,
        };

        let mut arguments = vec![];
        for input in self.sig.inputs.iter().skip(1) {
            arguments.push(input.convert(())?)
        }

        //TODO iter through params for codegen
        // Check if there is a returned value
        let returns = match self.sig.output {
            ReturnType::Default => false,
            ReturnType::Type(_, _) => true,
        };

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
                        mutability,
                        returns,
                        arguments,
                    }),
                    // Allow unreachable patters for future evolutions
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

impl<'a> ConvertToAst<()> for &'a FnArg {
    type Target = ast::MethodArgument;

    fn convert(self, _: ()) -> Result<Self::Target, Diagnostic> {
        match self {
            FnArg::Typed(pat_type) => {
                let (mutable, name) = match pat_type.pat.as_ref() {
                    Pat::Ident(i) => (i.mutability.is_some(), i.ident.to_string()),
                    _ => {
                        return Err(Diagnostic::error(format!(
                            "{}",
                            ExpectedBindingToNewVariable
                        )))
                    }
                };

                let arg_type = pat_type.ty.as_ref().convert(())?;

                Ok(ast::MethodArgument {
                    name,
                    mutable,
                    arg_type,
                })
            }
            FnArg::Receiver(_) => {
                return Err(Diagnostic::error(format!("{}", UnexpectedArgReceiver)))
            }
        }
    }
}

impl<'a> ConvertToAst<()> for &'a Type {
    type Target = syn::Member;

    fn convert(self, _: ()) -> Result<Self::Target, Diagnostic> {
        match self {
            Type::Array(a) => {
                let _ = a.elem.as_ref().convert(())?;

                Ok(syn::Member::Named(Ident::new(
                    &a.to_token_stream().to_string(),
                    Span::call_site(),
                )))
            }
            Type::Paren(p) => {
                let _ = p.elem.as_ref().convert(())?;

                Ok(syn::Member::Named(Ident::new(
                    &p.to_token_stream().to_string(),
                    Span::call_site(),
                )))
            }
            Type::Path(p) => {
                match &p.path.segments.last().unwrap().arguments {
                    PathArguments::None => {}
                    PathArguments::AngleBracketed(b) => {
                        for arg in b.args.iter() {
                            match arg {
                                GenericArgument::Lifetime(_) => {
                                    return Err(Diagnostic::error(format!(
                                        "{}",
                                        UnexpectedArgType(
                                            String::from("a type with specified lifetime"),
                                            p.to_token_stream().to_string()
                                        )
                                    )))
                                }
                                GenericArgument::Type(t) => {
                                    let _ = t.convert(())?;
                                }
                                GenericArgument::Binding(b) => {
                                    let _ = &b.ty.convert(())?;
                                }
                                GenericArgument::Constraint(_) => {
                                    return Err(Diagnostic::error(format!(
                                        "{}",
                                        UnexpectedArgType(
                                            String::from("a constraint type"),
                                            p.to_token_stream().to_string()
                                        )
                                    )))
                                }
                                GenericArgument::Const(_) => {
                                    return Err(Diagnostic::error(format!(
                                        "{}",
                                        UnexpectedArgType(
                                            String::from("a const expression"),
                                            p.to_token_stream().to_string()
                                        )
                                    )))
                                }
                            }
                        }
                    }
                    PathArguments::Parenthesized(_) => {
                        return Err(Diagnostic::error(format!(
                            "{}",
                            UnexpectedArgType(
                                String::from("arguments of a function path segment"),
                                p.to_token_stream().to_string()
                            )
                        )))
                    }
                }
                Ok(syn::Member::Named(Ident::new(
                    &p.to_token_stream().to_string(),
                    Span::call_site(),
                )))
            }
            Type::Tuple(t) => {
                for elem in t.elems.iter() {
                    let _ = elem.convert(())?;
                }

                Ok(syn::Member::Named(Ident::new(
                    &t.to_token_stream().to_string(),
                    Span::call_site(),
                )))
            }
            Type::BareFn(b) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("a bare function type"),
                    b.to_token_stream().to_string()
                )
            ))),
            Type::Group(g) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("a type contained within invisible delimiters"),
                    g.to_token_stream().to_string()
                )
            ))),
            Type::ImplTrait(i) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("an impl type"),
                    i.to_token_stream().to_string()
                )
            ))),
            Type::Infer(i) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("the infer type"),
                    i.to_token_stream().to_string()
                )
            ))),
            Type::Macro(m) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(String::from("a macro"), m.to_token_stream().to_string())
            ))),
            Type::Never(n) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("the never type"),
                    n.to_token_stream().to_string()
                )
            ))),
            Type::Ptr(p) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("a pointer type"),
                    p.to_token_stream().to_string()
                )
            ))),
            Type::Reference(r) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("a referenced type"),
                    r.to_token_stream().to_string()
                )
            ))),
            Type::Slice(s) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("a slice type"),
                    s.to_token_stream().to_string()
                )
            ))),
            Type::TraitObject(t) => Err(Diagnostic::error(format!(
                "{}",
                UnexpectedArgType(
                    String::from("a trait object type"),
                    t.to_token_stream().to_string()
                )
            ))),
            Type::Verbatim(v) => Err(Diagnostic::error(format!(
                "{}",
                UnhandledType(v.to_string())
            ))),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actor::attrs::ActorAttrs;
    use crate::utils::MacroParse;
    use backend::ast::Mutability;
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
                pub fn add(&mut self, value: u64) {
                    self.count += value
                }

                #[fvm_export(binding=3)]
                pub fn read(&self) -> u64 {
                    self.count
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

        let actor_entry_points = &program.actor_implementation.unwrap().entry_points;
        assert_eq!(actor_entry_points.len(), 3);

        assert_eq!(actor_entry_points[0].name, String::from("new"));
        assert_eq!(actor_entry_points[0].binding, Binding::Numeric(1));
        match actor_entry_points[0].mutability {
            Mutability::Pure => {}
            _ => panic!("method with no receiver should be pure"),
        }
        assert!(actor_entry_points[0].returns);

        assert_eq!(actor_entry_points[1].name, String::from("add"));
        assert_eq!(actor_entry_points[1].binding, Binding::Numeric(2));
        match actor_entry_points[1].mutability {
            Mutability::Write => {}
            _ => panic!("method with mutable receiver should be write"),
        }
        assert!(!actor_entry_points[1].returns);

        assert_eq!(actor_entry_points[2].name, String::from("read"));
        assert_eq!(actor_entry_points[2].binding, Binding::Numeric(3));
        match actor_entry_points[2].mutability {
            Mutability::View => {}
            _ => panic!("method with receiver should be view"),
        }
        assert!(actor_entry_points[2].returns);
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
        } else {
            panic!("method with #[fvm_export] and no attribute should throw an error")
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
        } else {
            panic!("method with #[fvm_export] and invalid value for binding with #[fvm_export] should throw an error")
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
        } else {
            panic!("unknown attribute on #[fvm_export] should throw an error")
        }
    }

    #[test]
    fn method_not_public() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export(binding=1)]
                fn new() -> Self {
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
                "compile_error ! { \"'new' can not be used as an entry point. Methods with #[fvm_export] should be public.\" }"
            )
        } else {
            panic!("non public method with #[fvm_export] should throw an error")
        }
    }

    #[test]
    fn generic_on_method() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export(binding=1)]
                fn mock<T: Sized>() -> T {
                    T::new()
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
                "compile_error ! { \"'mock' can not be used as an entry point. Methods with #[fvm_export] cannot have lifetime or type parameters.\" }"
            )
        } else {
            panic!("method with generics and #[fvm_export] should throw an error")
        }
    }
}
