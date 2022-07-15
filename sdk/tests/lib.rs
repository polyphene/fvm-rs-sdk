use fvm_rs_sdk::hello_world;

#[test]
fn assert_hello_world() {
    assert_eq!("Hello World!", hello_world())
}
struct Mock {
    pub count: u64
}

impl fvm_rs_sdk::state::StateObject for Mock {
fn load() -> Self {
    use fvm_rs_sdk::state::Blockstore;


    // First, load the current state root.
    let root = match fvm_rs_sdk::syscall::sself::root() {
        Ok(root) => root,
        Err(err) => fvm_rs_sdk::syscall::vm::abort(
            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
            Some(format!("failed to get root: {:?}", err).as_str()),
        ),
    };

    // Get state's bytes
    match fvm_rs_sdk::state::cbor::CborBlockstore.get(&root) {
        // State fetched, convert byte to actual struct
        Ok(Some(state_bytes)) => match fvm_rs_sdk::encoding::from_slice(&state) {
            Ok(state) => state,
            Err(err) => fvm_rs_sdk::syscall::vm::abort(
                fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
                Some(format!("failed to deserialize state: {}", err).as_str()),
            )
        },
        // No state
        Ok(None) => fvm_rs_sdk::syscall::vm::abort(
            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
            Some("state does not exist"),
        ),
        // Fetching failed
        Err(err) => fvm_rs_sdk::syscall::vm::abort(
            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
            Some(format!("failed to get state: {}", err).as_str()),
        ),
    }
}

fn save(&self) -> fvm_rs_sdk::cid::Cid {
    use fvm_rs_sdk::state::Blockstore;

    // Serialize state
    let serialized = match fvm_rs_sdk::encoding::to_vec(self) {
        Ok(s) => s,
        Err(err) => fvm_rs_sdk::syscall::vm::abort(
            fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
            Some(format!("failed to serialize state: {:?}", err).as_str()),
        ),
    };

    // Put state
    let cid = fvm_rs_sdk::state::cbor::CborBlockstore.put(fvm_rs_sdk::cid::Code::Blake2b256.into())
    let cid = match fvm_rs_sdk::state::cbor::CborBlockstore.put(
        fvm_rs_sdk::cid::Code::Blake2b256.into(),
        &fvm_rs_sdk::state::Block {
            codec: fvm_rs_sdk::encoding::DAG_CBOR,
            data: serialized.as_slice()
        }
    ) {
        Ok(cid) => cid,
        Err(err) => fvm_rs_sdk::syscall::vm::abort(
            fvm_rs_sdk::shared::error::ExitCode::USR_SERIALIZATION.value(),
            Some(format!("failed to store initial state: {:}", err).as_str()),
        ),
    };
    if let Err(err) = fvm_rs_sdk::syscall::sself::set_root(&cid) {
        fvm_rs_sdk::syscall::vm::abort(
            fvm_rs_sdk::shared::error::ExitCode::USR_ILLEGAL_STATE.value(),
            Some(format!("failed to set root cid: {:}", err).as_str()),
        );
    }
    cid
}
}