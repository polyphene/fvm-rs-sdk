//! Codegen has the logic of code generation for our actor through the `#[fvm_state]` macro.

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::convert::TryInto;

use crate::ast;
use crate::ast::Mutability;

impl ToTokens for ast::ActorImplementation {
    fn to_tokens(&self, into: &mut TokenStream) {
        let impl_member = self.rust_name.clone();
        let mut entry_points: Vec<TokenStream> = vec![];

        for entry_point in self.entry_points.iter() {
            let entry_point_value: u64 = entry_point.binding.clone().try_into().unwrap();
            let method_name = entry_point.rust_name.clone();

            // Token stream to represent variables in which we will deserialize
            let mut parameters_variables = TokenStream::new();
            // Token stream representing the parameters passed along the method
            let mut method_parameters = TokenStream::new();
            // Token stream representing the types of the variables, for deserialization
            let mut parameters_types = TokenStream::new();
            // Token stream representing the code to fetch & deserialize parameters
            let mut parameters_deserialization = TokenStream::new();

            // If there are parameters for the method then prepare them for the call
            if entry_point.arguments.len() > 0usize {
                for (i, argument) in entry_point.arguments.iter().enumerate() {
                    let arg_type = argument.arg_type.clone();

                    // Variable name based on argument name & index, to prevent naming collision
                    let variable = syn::Member::Named(Ident::new(
                        &format!("{}{}", argument.name, i),
                        Span::call_site(),
                    ));
                    // Add variable type to token stream
                    quote!(#arg_type).to_tokens(&mut parameters_types);

                    // If argument has to be mutable pass variable name with `mut`
                    if argument.mutable {
                        quote!(mut #variable).to_tokens(&mut parameters_variables);
                    } else {
                        quote!(#variable).to_tokens(&mut parameters_variables);
                    }
                    // Pass variable name in method parameters
                    quote!(#variable).to_tokens(&mut method_parameters);

                    // If not the last entry, add comma
                    if i != entry_point.arguments.len() - 1 {
                        quote!(, ).to_tokens(&mut parameters_types);
                        quote!(, ).to_tokens(&mut parameters_variables);
                        quote!(, ).to_tokens(&mut method_parameters);
                    }
                }
                // Code to fetch bytes from pointer then deserialize in given variables
                quote!(
                    let params_bytes = fvm_rs_sdk::syscall::message::params_raw(params_pointer).unwrap().1;
                    let (#parameters_variables): (#parameters_types) = fvm_rs_sdk::encoding::RawBytes::new(params_bytes).deserialize().unwrap();
                )
                .to_tokens(&mut parameters_deserialization);
            }

            let mut method_call = TokenStream::new();

            // If method not pure load state
            if !matches!(entry_point.mutability, Mutability::Pure) {
                // let keyword
                quote!(let).to_tokens(&mut method_call);
                if matches!(entry_point.mutability, Mutability::Write) {
                    // Add mut keyword if write mutability
                    quote!(mut).to_tokens(&mut method_call);
                }
                // Finalize load() call
                quote!(
                    state = #impl_member::load();
                )
                .to_tokens(&mut method_call);
            }

            // If method returns then store method result in variable
            if entry_point.returns {
                quote!(
                    let method_return =
                )
                .to_tokens(&mut method_call);
            }

            // Handle method calling based on mutability
            match entry_point.mutability {
                Mutability::Pure => quote!(
                    #impl_member::#method_name(#method_parameters);
                )
                .to_tokens(&mut method_call),
                _ => quote!(
                    state.#method_name(#method_parameters);
                )
                .to_tokens(&mut method_call),
            };

            // If method returns then convert result to bytes
            if entry_point.returns {
                quote!(
                    ret = match(fvm_rs_sdk::encoding::to_vec(&method_return)) {
                        Ok(ret) => Some(fvm_rs_sdk::encoding::RawBytes::new(ret)),
                        Err(err) => {
                            fvm_rs_sdk::syscall::vm::abort(
                                fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
                                Some(format!("failed to serialize return value: {:?}", err).as_str()),
                            );
                        }
                    };
                ).to_tokens(&mut method_call)
            }

            // If mutability is write then save state
            if matches!(entry_point.mutability, Mutability::Write) {
                quote!(
                    state.save();
                )
                .to_tokens(&mut method_call);
            }

            entry_points.push(quote!(
                #entry_point_value => {
                    #parameters_deserialization
                    let mut ret = None;
                    #method_call
                    ret
                }
            ));
        }

        quote!(
            #[no_mangle]
            pub fn invoke(params_pointer: u32) -> u32 {
                // Conduct method dispatch. Handle input parameters and return data.
                let ret: Option<fvm_rs_sdk::encoding::RawBytes> =
                    match fvm_rs_sdk::syscall::message::method_number() {
                        #(#entry_points),*
                        _ => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_UNHANDLED_MESSAGE.value(),
                            Some("unrecognized method"),
                        ),
                    };

                match ret {
                    None => fvm_rs_sdk::syscall::NO_DATA_BLOCK_ID,
                    Some(v) => match fvm_rs_sdk::syscall::ipld::put_block(
                        fvm_rs_sdk::encoding::DAG_CBOR,
                        v.bytes(),
                    ) {
                        Ok(id) => id,
                        Err(err) => fvm_rs_sdk::syscall::vm::abort(
                            fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
                            Some(format!("failed to store return value: {}", err).as_str()),
                        ),
                    },
                }
            }
        )
        .to_tokens(into)
    }
}

#[cfg(test)]
mod tests {}
