use fvm_rs_sdk::actor::{fvm_actor, fvm_export};
use fvm_rs_sdk::state::*;

#[derive(Clone, Debug, Default)]
#[fvm_state]
pub struct State {
    pub value: u64,
}

#[fvm_actor]
impl State {
    #[fvm_export(binding = 1)]
    pub fn new() -> Self {
        State { value: 0 }
    }

    #[fvm_export(binding = 2)]
    pub fn add(&mut self, value: u64) {
        self.value += value
    }

    #[fvm_export(binding = 3)]
    pub fn read(&self) -> u64 {
        self.value
    }
}
