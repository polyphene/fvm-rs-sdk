pub mod state;

pub use {
    fvm_ipld_encoding as encoding, fvm_sdk as syscall, fvm_shared as shared, serde, serde_tuple,
};

pub mod cid {
    pub use cid::multihash::Code;
    pub use cid::Cid;
}

pub fn hello_world() -> &'static str {
    "Hello World!"
}
