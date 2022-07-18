# Filecoin Virtual Machine Rust SDK

This repository contains the implementation of a Rust SDK to build actor for the Filecoin Virtual Machine (FVM). It exposes
useful structures and functions that could be needed by developers while also serving procedural macros to generate
glue code for state management and actor's interface definition.

## Code structure

Here is a quick description of what you will be able to find in every directory:
- `sdk`
  - The entry point for the Rust SDK. The key concepts are:
    - `StateObject`: a trait that contains logic needed to handle read and write on the FVM state. A standard Cbor 
    implementation is available.
- `macro`
  - The procedural macro crate, responsible to expose the `fvm_state`, `fvm_actor` and `fvm_export` procedural macros.
- `macro-support`
  - This crate contains the logic for parsing Rust code tokens to usable structures that we will use to generate glue code.
  A dedicated parser is available for each of our procedural macros.
- `backend`
  - The `backend` crate is dedicated to two things: representing Rust code with dedicated structures and generating glue 
  code to ensure proper state management and interface definition for actors.
    - Structures used to represent the actor's code can be found in the `ast` module.
    - Each procedural macro have its own generation logic in their dedicated module (e.g. `state` for `fvm_state`).

## License

Dual-licensed: [MIT](./LICENSE-MIT), [Apache Software License v2](./LICENSE-APACHE), by way of the
[Permissive License Stack](https://protocol.ai/blog/announcing-the-permissive-license-stack/).
