//! Parser reads a source `TokenStream` to prepare the backend to generate custom code

use crate::payload::attrs::PayloadAttrs;
use backend::payload::attrs::Codec;
use backend::{ast, Diagnostic};
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::payload::error::Error::{ExpectedStructure, GenericsOnStructure};
use crate::utils::{ConvertToAst, MacroParse};

impl<'a> ConvertToAst<PayloadAttrs> for &'a mut syn::ItemStruct {
    type Target = ast::PayloadStruct;

    fn convert(self, attrs: PayloadAttrs) -> Result<Self::Target, Diagnostic> {
        // No lifetime to make sure that we can handle it correctly
        if !self.generics.params.is_empty() {
            return Err(Diagnostic::error(format!("{}", GenericsOnStructure)));
        }

        // Attrs assignment
        let codec = match attrs.codec() {
            Some(codec) => codec.clone(),
            None => Codec::default(),
        };

        // Generate the AST object for the Struct
        Ok(ast::PayloadStruct {
            rust_name: self.ident.to_token_stream(),
            name: self.ident.to_string(),
            codec,
        })
    }
}

impl<'a> MacroParse<(Option<PayloadAttrs>, &'a mut TokenStream)> for syn::Item {
    fn macro_parse(
        self,
        program: &mut ast::Program,
        (attrs, tokens): (Option<PayloadAttrs>, &'a mut TokenStream),
    ) -> Result<(), Diagnostic> {
        // Match of Item types to parse & generate our AST
        match self {
            // Handles structures
            syn::Item::Struct(mut s) => {
                let attrs = attrs.unwrap_or_default();
                program.payload_structs.push((&mut s).convert(attrs)?);
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
    use backend::payload::attrs::Codec;
    use quote::quote;

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
        let attrs: PayloadAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let parsed_struct = &program.payload_structs[0];

        assert_eq!(parsed_struct.name, "MockStruct");
        assert_eq!(parsed_struct.rust_name.to_string(), parsed_struct.name);
        assert_eq!(parsed_struct.codec, Codec::DagCbor);
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
        let attrs: PayloadAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        let res_parse = item.macro_parse(&mut program, (Some(attrs), &mut tokens));

        match res_parse {
            Err(diagnostic) => {
                let res_panic = std::panic::catch_unwind(|| diagnostic.panic());
                match res_panic {
                    Err(err) => match err.downcast::<String>() {
                        Ok(panic_msg_box) => {
                            assert_eq!(panic_msg_box.as_str(), "structure with #[fvm_payload] cannot have lifetime or type parameters.");
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
        let attrs: PayloadAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let parsed_struct = &program.payload_structs[0];

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
        match syn::parse2::<PayloadAttrs>(attrs_token_stream) {
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
        match syn::parse2::<PayloadAttrs>(attrs_token_stream) {
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
        match syn::parse2::<PayloadAttrs>(attrs_token_stream) {
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
        let attrs: PayloadAttrs = syn::parse2(attrs_token_stream).unwrap();

        let mut tokens = TokenStream::new();
        let mut program = backend::ast::Program::default();

        item.macro_parse(&mut program, (Some(attrs), &mut tokens))
            .unwrap();

        let parsed_struct = &program.payload_structs[0];

        assert_eq!(parsed_struct.codec, Codec::DagCbor)
    }
}
