//! Codegen has the logic of code generation for our actor through the `#[fvm_state]` macro.

use proc_macro2::TokenStream;
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

            let mut method_call = TokenStream::new();

            // If method not pure load state
            if !entry_point.mutability.is_pure() {
                // let keyword
                quote!(let).to_tokens(&mut method_call);
                if entry_point.mutability.is_write() {
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
                    #impl_member::#method_name();
                )
                .to_tokens(&mut method_call),
                Mutability::View => quote!(
                    state.#method_name();
                )
                .to_tokens(&mut method_call),
                Mutability::Write => quote!(
                    state.#method_name();
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
            if entry_point.mutability.is_write() {
                quote!(
                    state.save();
                )
                .to_tokens(&mut method_call);
            }

            entry_points.push(quote!(
                #entry_point_value => {
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
