macro_rules! generate_attrs {
    ($struct_name:ident, $attrs_enum:path) => {
        #[derive(Debug, Default)]
        pub struct $struct_name {
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
