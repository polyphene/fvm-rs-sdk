//! Backend crate for the Filecoin Virtual Machine Rust SDK procedural macro.

pub use state::codegen::TryToTokens as TryStateToTokens;

pub use crate::error::Diagnostic;

#[macro_use]
mod error;
pub mod ast;
pub mod state;
