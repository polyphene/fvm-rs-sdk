#![allow(unreachable_code)]
use fvm_rs_sdk::payload::*;

#[fvm_payload]
pub struct MockStruct1<'a> {
    pub count: &'a u64,
}

#[fvm_payload]
pub struct MockStruct2 {
    pub inner: InnerStruct,
}

struct InnerStruct {
    pub count: u64,
}

fn main() {}
