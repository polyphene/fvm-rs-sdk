use fvm_ipld_encoding::CborStore;
use fvm_rs_sdk::hello_world;
use fvm_rs_sdk_macro::fvm_state;

#[test]
fn assert_hello_world() {
    assert_eq!("Hello World!", hello_world())
}
