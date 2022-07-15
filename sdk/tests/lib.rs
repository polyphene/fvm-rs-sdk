use fvm_rs_sdk::hello_world;
use fvm_rs_sdk_macro::fvm_state;

#[test]
fn assert_hello_world() {
    assert_eq!("Hello World!", hello_world())
}

#[test]
fn mock() {
    #[fvm_state]
    pub struct Mock {
        pub count: u64,
    }

    assert!(false)
}
