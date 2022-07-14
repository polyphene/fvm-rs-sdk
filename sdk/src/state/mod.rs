//! State contains necessary code to handle a state object in an actor
mod cbor;

use cid::Cid;
pub use fvm_rs_sdk_macro::fvm_state;

/// StateObject is a trait to read and write an actor's state on the Filecoin Virtual Machine
trait StateObject {
    // Load state object from the FVM state
    fn load() -> Self;
    // Save object as an actor's state
    fn save(&self) -> Cid;
}
