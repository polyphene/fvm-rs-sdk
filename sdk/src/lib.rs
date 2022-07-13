pub use fvm_rs_sdk_macro::fvm_state;
pub use {fvm_sdk as syscall, fvm_shared as shared};

pub fn hello_world() -> &'static str {
    "Hello World!"
}
