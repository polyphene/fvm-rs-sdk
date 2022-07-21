use fvm_rs_sdk::encoding::{to_vec, RawBytes, DAG_CBOR};
use fvm_rs_sdk::shared::error::ExitCode;
use fvm_rs_sdk::syscall::{message::params_raw, vm::abort, NO_DATA_BLOCK_ID};

use fvm_rs_sdk::state::*;

#[derive(Clone, Debug, Default)]
#[fvm_state]
pub struct State {
    pub value: u64,
}

#[no_mangle]
pub fn invoke(params_pointer: u32) -> u32 {
    // Conduct method dispatch. Handle input parameters and return data.
    let ret: Option<RawBytes> = match fvm_rs_sdk::syscall::message::method_number() {
        // Set initial value
        1 => {
            let params = params_raw(params_pointer).unwrap().1;
            let x: u64 = RawBytes::new(params).deserialize().unwrap();

            let mut state = State::load();
            state.value = x;
            state.save();

            None
        }
        // Add value
        2 => {
            let params = params_raw(params_pointer).unwrap().1;
            let x: u64 = RawBytes::new(params).deserialize().unwrap();

            let mut state = State::load();
            state.value += x;
            state.save();

            None
        }
        // Get state value
        3 => {
            let state = State::load();
            let ret = to_vec(&state.value);
            match ret {
                Ok(ret) => Some(RawBytes::new(ret)),
                Err(err) => {
                    abort(
                        ExitCode::USR_ILLEGAL_STATE.value(),
                        Some(format!("failed to serialize return value: {:?}", err).as_str()),
                    );
                }
            }
        }
        _ => abort(
            ExitCode::USR_UNHANDLED_MESSAGE.value(),
            Some("unrecognized method"),
        ),
    };

    match ret {
        None => NO_DATA_BLOCK_ID,
        Some(v) => match fvm_rs_sdk::syscall::ipld::put_block(DAG_CBOR, v.bytes()) {
            Ok(id) => id,
            Err(err) => abort(
                ExitCode::USR_SERIALIZATION.value(),
                Some(format!("failed to store return value: {}", err).as_str()),
            ),
        },
    }
}
