#![allow(unreachable_code)]
use fvm_rs_sdk::actor::fvm_export;
use fvm_rs_sdk::state::*;

#[fvm_state]
pub struct MockStruct1 {
    pub count: u64,
}

// Test path import for fvm_actor
#[fvm_rs_sdk::actor::fvm_actor]
impl MockStruct1 {
    // Test mutable state method & different types
    #[fvm_export(method_num = 1)]
    pub fn first_mock(
        &mut self,
        _a: u64,
        _b: Vec<u8>,
        _c: fvm_rs_sdk::cid::Cid,
        _d: [u8; 5],
        _e: (String, String),
    ) -> u64 {
        0
    }
    // Test read state method
    #[fvm_export(method_num = 2)]
    pub fn second_mock(&self) -> u64 {
        0
    }
    // Test pure method
    #[fvm_export(method_num = 3)]
    pub fn third_mock() -> u64 {
        0
    }

    // Test path import for fvm_export
    #[fvm_rs_sdk::actor::fvm_export(method_num = 4)]
    pub fn fourth_mock() -> u64 {
        0
    }
}

fn main() {}
