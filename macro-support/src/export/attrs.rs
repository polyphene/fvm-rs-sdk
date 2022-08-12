use crate::utils::{generate_attr_getters, generate_attrs};
use backend::export::attrs::{Binding, ExportAttr};
use syn::parse::{Parse, ParseStream, Result};

generate_attrs!(ExportAttrs, ExportAttr);

generate_attr_getters!(ExportAttrs, [(binding, ExportAttr::Binding, Binding),]);
