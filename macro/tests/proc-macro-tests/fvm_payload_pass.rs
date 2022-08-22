#![allow(unreachable_code)]
use fvm_rs_sdk::payload::*;

#[fvm_payload]
pub struct MockStruct1 {
    pub count: u64,
}

#[fvm_payload(codec = "dag-cbor")]
pub struct MockStruct2 {
    pub count: u64,
}

fn main() {}
