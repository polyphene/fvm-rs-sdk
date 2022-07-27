#![allow(unreachable_code)]
use fvm_rs_sdk::state::*;

#[fvm_state]
pub struct MockStruct1 {
    pub count: u64,
}

#[fvm_state(codec = "dag-cbor")]
pub struct MockStruct {
    pub count: u64,
}

fn main() {}
