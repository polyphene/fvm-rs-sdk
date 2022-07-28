//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use backend::state::attrs::Codec;
use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::state::attrs::StateAttrs;
use crate::state::error::Error::{ExpectedStructure, GenericsOnStructure};
use crate::utils::{ConvertToAst, MacroParse};

impl<'a> ConvertToAst<StateAttrs> for &'a mut syn::ItemStruct {
    type Target = ast::StateStruct;

    fn convert(self, attrs: StateAttrs) -> Result<Self::Target, Diagnostic> {
        // No lifetime to make sure that we can handle it correctly
        if !self.generics.params.is_empty() {
            return Err(Diagnostic::error(format!("{}", GenericsOnStructure)));
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

            fields.push(ast::StateStructField {
                rust_name: member,
                name,
                struct_name: self.ident.clone(),
                ty: field.ty.clone(),
            });
        }

        // Attrs assignment
        let codec = match attrs.codec() {
            Some(codec) => codec.clone(),
            None => Codec::default(),
        };

        // Generate the AST object for the Struct
        Ok(ast::StateStruct {
            rust_name: self.ident.clone(),
            name: self.ident.to_string(),
            fields,
            codec,
        })
    }
}

impl<'a> MacroParse<(Option<StateAttrs>, &'a mut TokenStream)> for syn::Item {
    fn macro_parse(
        self,
        program: &mut ast::Program,
        (attrs, tokens): (Option<StateAttrs>, &'a mut TokenStream),
    ) -> Result<(), Diagnostic> {
        // Match of Item types to parse & generate our AST
        match self {
            // Handles structures
            syn::Item::Struct(mut s) => {
                let attrs = attrs.unwrap_or_default();
                program.state_structs.push((&mut s).convert(attrs)?);
                s.to_tokens(tokens);
            }
            _ => {
                return Err(Diagnostic::error(format!("{}", ExpectedStructure)));
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
    fn struct_to_ast() {
        // Mock struct token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            pub struct MockStruct {
                pub count: u64
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Mock no attrs
        let attrs_token_stream = TokenStream::new();

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: StateAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let parsed_struct = &program.state_structs[0];
        let parsed_field = &parsed_struct.fields[0];

        assert_eq!(parsed_struct.name, "MockStruct");
        assert_eq!(parsed_struct.rust_name.to_string(), parsed_struct.name);
        assert_eq!(parsed_struct.codec, Codec::DagCbor);

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
        // Mock struct token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            pub struct MockStruct<'a> {
                pub count: &'a u64
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Mock no attrs
        let attrs_token_stream = TokenStream::new();

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: StateAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        let res_parse = item.macro_parse(&mut program, (Some(attrs), &mut tokens));

        match res_parse {
            Err(diagnostic) => {
                let res_panic = std::panic::catch_unwind(|| diagnostic.panic());
                match res_panic {
                    Err(err) => match err.downcast::<String>() {
                        Ok(panic_msg_box) => {
                            assert_eq!(panic_msg_box.as_str(), "structure with #[fvm_state] cannot have lifetime or type parameters.");
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
        // Mock struct token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            pub struct MockStruct {
                pub count: u64,
                count_bis: u64
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Mock no attrs
        let attrs_token_stream = TokenStream::new();

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: StateAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let parsed_struct = &program.state_structs[0];
        let parsed_field = &parsed_struct.fields[0];

        assert_eq!(parsed_struct.fields.len(), 1usize);
        assert_eq!(parsed_field.name, "count");
    }

    #[test]
    fn struct_with_codec_attr() {
        // Mock struct token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            pub struct MockStruct {
                pub count: u64
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Mock no attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {
            codec = "dag-cbor"
        })
        .to_tokens(&mut attrs_token_stream);

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: StateAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let parsed_struct = &program.state_structs[0];

        assert_eq!(parsed_struct.codec, Codec::DagCbor)
    }

    #[test]
    fn struct_with_unknown_attr() {
        // Mock no attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {
            john = "dag-cbor"
        })
        .to_tokens(&mut attrs_token_stream);

        // Parse struct and attrs
        match syn::parse2::<StateAttrs>(attrs_token_stream) {
            Err(err) => assert_eq!(err.to_string(), "unknown attribute 'john'"),
            _ => panic!("unknown attribute should throw an error"),
        }
    }

    #[test]
    fn struct_with_unknown_codec() {
        // Mock no attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {
            codec = "john"
        })
        .to_tokens(&mut attrs_token_stream);

        // Parse struct and attrs
        match syn::parse2::<StateAttrs>(attrs_token_stream) {
            Err(err) => assert_eq!(err.to_string(), "unknown codec 'john'"),
            _ => panic!("unknown attribute should throw an error"),
        }
    }

    #[test]
    fn struct_with_invalid_codec_format() {
        // Mock no attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {
            codec = dag-cbor
        })
        .to_tokens(&mut attrs_token_stream);

        // Parse struct and attrs
        match syn::parse2::<StateAttrs>(attrs_token_stream) {
            Err(err) => assert_eq!(
                err.to_string(),
                "invalid codec format, expected string literal"
            ),
            _ => panic!("invalid codec format should throw an error"),
        }
    }

    #[test]
    fn struct_with_no_attrs() {
        // Mock no attrs
        let mut attrs_token_stream = TokenStream::new();
        (quote! {}).to_tokens(&mut attrs_token_stream);

        // Mock struct token stream
        let mut struct_token_stream = TokenStream::new();

        (quote! {
            pub struct MockStruct {
                pub count: u64
            }
        })
        .to_tokens(&mut struct_token_stream);

        // Parse struct and attrs
        let item = syn::parse2::<syn::Item>(struct_token_stream).unwrap();
        let attrs: StateAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let parsed_struct = &program.state_structs[0];

        assert_eq!(parsed_struct.codec, Codec::DagCbor)
    }
}
