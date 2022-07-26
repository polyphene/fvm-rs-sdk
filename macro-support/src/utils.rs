use backend::{ast, Diagnostic};

/// Conversion trait with context.
///
/// Used to convert syn tokens into an AST, that we can then use to generate glue code.
pub(crate) trait ConvertToAst<Ctx> {
    /// What we are converting to.
    type Target;
    /// Convert into our target.
    ///
    /// Since this is used in a procedural macro, use panic to fail.
    fn convert(self, context: Ctx) -> Result<Self::Target, Diagnostic>;
}

pub(crate) trait MacroParse<Ctx> {
    /// Parse the contents of an object into our AST, with a context if necessary.
    ///
    /// The context is used to have access to the attributes on the procedural macro, and to allow
    /// writing to the output `TokenStream`.
    fn macro_parse(self, program: &mut ast::Program, context: Ctx) -> Result<(), Diagnostic>;
}

macro_rules! generate_attrs {
    ($struct_name:ident, $attrs_enum:path) => {
        #[derive(Debug, Default)]
        pub(crate) struct $struct_name {
            pub attrs: Vec<$attrs_enum>,
        }

        impl Parse for $struct_name {
            fn parse(input: ParseStream) -> Result<Self> {
                let mut attrs = $struct_name::default();
                if input.is_empty() {
                    return Ok(attrs);
                }

                let opts =
                    syn::punctuated::Punctuated::<$attrs_enum, syn::token::Comma>::parse_terminated(input)?;
                attrs.attrs = opts.into_iter().collect();

                Ok(attrs)
            }
        }
    };
}

macro_rules! generate_attr_getters {
    ($struct_name:ident, [$(($getter_name:ident, $attr_value:path, $attr_ty:path),)*]) => {
        impl $struct_name {
            $(
                pub fn $getter_name(&self) -> Option<&$attr_ty> {
                    self.attrs
                        .iter()
                        .filter_map(|a| match &a {
                            $attr_value(value) => Some(value),
                        })
                        .next()
                }
            )*
        }
    };
}

pub(crate) use generate_attr_getters;
pub(crate) use generate_attrs;
