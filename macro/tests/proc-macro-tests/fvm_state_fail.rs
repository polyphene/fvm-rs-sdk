#![allow(unreachable_code)]
use fvm_rs_sdk::*;

#[fvm_state]
pub struct MockStruct1<'a> {
    pub count: &'a u64,
}

#[fvm_state]
pub struct MockStruct2 {
    pub inner: InnerStruct,
}

struct InnerStruct {
    pub count: u64,
}

fn main() {}
