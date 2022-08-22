//! Convert reads a source `TokenStream` to prepare the backend to generate custom code

use crate::utils::ConvertToAst;
use backend::ast::Mutability;
use backend::export::attrs::Method;
use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{FnArg, GenericArgument, Pat, PathArguments, ReturnType, Type};

use crate::export::attrs::ExportAttrs;
use crate::export::error::Error::{
    ExpectedBindingToNewVariable, GenericsOnEntryPoint, MissingBindingMethod,
    UnexpectedArgReceiver, UnexpectedArgType, UnhandledType,
};

impl<'a> ConvertToAst<ExportAttrs> for &'a mut syn::ImplItemMethod {
    type Target = ast::ActorEntryPoint;

    fn convert(self, attrs: ExportAttrs) -> Result<Self::Target, Diagnostic> {
        // Not handling generics on entry point
        if !self.sig.generics.params.is_empty() {
            return Err(Diagnostic::error(format!(
                "{}",
                GenericsOnEntryPoint(self.sig.ident.to_string())
            )));
        }

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

        // Check if there is a returned value
        let returns = match self.sig.output {
            ReturnType::Default => false,
            ReturnType::Type(_, _) => true,
        };

        // Trying to get a valid dispatch method and value
        let binding_method: &Method = attrs.binding_method().ok_or_else(|| {
            Diagnostic::error(format!(
                "{}",
                MissingBindingMethod(self.sig.ident.to_string())
            ))
        })?;

        match binding_method {
            Method::Numeric(value) => Ok(ast::ActorEntryPoint {
                rust_name: self.sig.ident.to_token_stream(),
                name: self.sig.ident.to_string(),
                binding: Method::Numeric(*value),
                mutability,
                returns,
                arguments,
            }),
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
            FnArg::Receiver(_) => Err(Diagnostic::error(format!("{}", UnexpectedArgReceiver))),
        }
    }
}

impl<'a> ConvertToAst<()> for &'a Type {
    type Target = TokenStream;

    fn convert(self, _: ()) -> Result<Self::Target, Diagnostic> {
        match self {
            Type::Array(a) => {
                let _ = a.elem.as_ref().convert(())?;

                Ok(a.to_token_stream())
            }
            Type::Paren(p) => {
                let _ = p.elem.as_ref().convert(())?;

                Ok(p.to_token_stream())
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
                Ok(p.to_token_stream())
            }
            Type::Tuple(t) => {
                for elem in t.elems.iter() {
                    let _ = elem.convert(())?;
                }

                Ok(t.to_token_stream())
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
    use crate::utils::MacroParse;
    use backend::ast::Mutability;
    use backend::export::attrs::Method;
    use proc_macro2::TokenStream;
    use quote::quote;
    use quote::ToTokens;

    #[test]
    fn export_to_ast() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export(method_num=1)]
                pub fn new() -> Self {
                    Actor {
                        count: 0
                    }
                }

                #[fvm_export(method_num=2)]
                pub fn add(&mut self, value: u64) {
                    self.count += value
                }

                #[fvm_export(method_num=3)]
                pub fn read(&self) -> u64 {
                    self.count
                }
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, &mut tokens).unwrap();

        let actor_entry_points = &program.actor_implementation.unwrap().entry_points;
        assert_eq!(actor_entry_points.len(), 3);

        assert_eq!(actor_entry_points[0].name, String::from("new"));
        assert_eq!(actor_entry_points[0].binding, Method::Numeric(1));
        match actor_entry_points[0].mutability {
            Mutability::Pure => {}
            _ => panic!("method with no receiver should be pure"),
        }
        assert!(actor_entry_points[0].returns);

        assert_eq!(actor_entry_points[1].name, String::from("add"));
        assert_eq!(actor_entry_points[1].binding, Method::Numeric(2));
        match actor_entry_points[1].mutability {
            Mutability::Write => {}
            _ => panic!("method with mutable receiver should be write"),
        }
        assert!(!actor_entry_points[1].returns);

        assert_eq!(actor_entry_points[2].name, String::from("read"));
        assert_eq!(actor_entry_points[2].binding, Method::Numeric(3));
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

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        if let Err(err) = item.macro_parse(&mut program, &mut tokens) {
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
                #[fvm_export(method_num="toto")]
                pub fn new() -> Self {
                    Actor {
                        count: 0
                    }
                }
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        if let Err(err) = item.macro_parse(&mut program, &mut tokens) {
            assert_eq!(
                err.to_token_stream().to_string(),
                "compile_error ! { \"invalid 'method_num' value\" }"
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

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        if let Err(err) = item.macro_parse(&mut program, &mut tokens) {
            assert_eq!(
                err.to_token_stream().to_string(),
                "compile_error ! { \"unknown attribute 'hello'\" }"
            )
        } else {
            panic!("unknown attribute on #[fvm_export] should throw an error")
        }
    }

    #[test]
    fn generic_on_method() {
        // Mock impl token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            impl Actor {
                #[fvm_export(method_num=1)]
                fn mock<T: Sized>() -> T {
                    T::new()
                }
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        if let Err(err) = item.macro_parse(&mut program, &mut tokens) {
            assert_eq!(
                err.to_token_stream().to_string(),
                "compile_error ! { \"'mock' can not be used as an entry point. Methods with #[fvm_export] cannot have lifetime or type parameters.\" }"
            )
        } else {
            panic!("method with generics and #[fvm_export] should throw an error")
        }
    }
}
