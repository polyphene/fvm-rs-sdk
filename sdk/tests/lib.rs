use fvm_rs_sdk::hello_world;

#[test]
fn assert_hello_world() {
    assert_eq!("Hello World!", hello_world())
}
