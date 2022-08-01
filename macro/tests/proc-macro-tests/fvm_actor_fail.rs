#![allow(unreachable_code)]
use fvm_rs_sdk::actor::{fvm_actor, fvm_export};
use fvm_rs_sdk::state::*;

#[fvm_state]
pub struct MockStruct1 {
    pub count: u64,
}

// Fail because of generic
#[fvm_actor]
impl<T> MockStruct1 {
    #[fvm_export(binding = 1)]
    fn add(&mut self, a: T) {
        self.count += T.count
    }
}

// Fail because of ref
#[fvm_actor]
impl MockStruct1 {
    #[fvm_export(binding = 1)]
    pub fn add(&mut self, a: &u64) {
        self.count += a;
    }
}

// Fail because of bare function type
#[fvm_actor]
impl MockStruct1 {
    #[fvm_export(binding = 1)]
    pub fn add(&mut self, a: fn(u64) -> u64) {
        self.count += a
    }
}

// Fail because no binding
#[fvm_actor]
impl MockStruct1 {
    #[fvm_export]
    pub fn add(&mut self, a: u64) {
        self.count += a;
    }
}

// Fail because pointer type
#[fvm_actor]
impl MockStruct1 {
    #[fvm_export(binding = 1)]
    pub fn add(&mut self, a: *mut u64) {
        todo!()
    }
}

// Fail because private method
#[fvm_actor]
impl MockStruct1 {
    #[fvm_export(binding = 1)]
    fn add(&mut self, a: u64) {
        self.count += a;
    }
}

// Fail because never argument type
#[fvm_actor]
impl MockStruct1 {
    #[fvm_export(binding = 1)]
    pub fn call_never(&mut self, a: !) {
        todo!()
    }
}

// Fail because slice argument type
#[fvm_actor]
impl MockStruct1 {
    #[fvm_export(binding = 1)]
    pub fn call_never(&mut self, a: Box<[u64]>) {
        self.count += a[0]
    }
}

fn main() {}
