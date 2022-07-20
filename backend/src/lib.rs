//! Backend crate for the Filecoin Virtual Machine Rust SDK procedural macro.

pub use crate::error::Diagnostic;
pub use crate::state_codegen::TryToTokens as TryStateToTokens;

#[macro_use]
mod error;
pub mod ast;
mod state_codegen;
