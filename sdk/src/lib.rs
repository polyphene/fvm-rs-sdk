mod state;

pub use {fvm_sdk as syscall, fvm_shared as shared, serde, serde_tuple};

pub fn hello_world() -> &'static str {
    "Hello World!"
}
