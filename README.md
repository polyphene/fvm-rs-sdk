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

## Usage

### Limitations 

As of now some references to `ref-fvm` are done through [cargo patches]() to ease the development of the SDK. This 
reference makes the SDK harder to use and to test through integration in a Filecoin Virtual Machine. However, the final
version will be simply a line in your dependencies so please bear with us until then!

### Pre-requirements

Before using the SDK, clone this repository on your local machine and make sure to initialize and update the `ref-fvm` 
submodule.

```bash
git submodule init
git submodule update
```
 
### Build an actor using the SDK

To build an actor using the SDK you can follow the example of the `sdk_example_actor` crate in `./sdk/tests/sdk-example-actor`.
Once your actor's crate is ready just run:
```bash
cargo build
```

Using `wasm-builder` in your `build.rs` file will compile the crate in a wasm file that can be found at the path 
`./target/debug/wbuild/<crate-name>/<crate-name>.compact.wasm`.

### Test your actor

You can test your actor by either integrate it in the `fvm_integration_tests` crate in `ref-fvm` or by running your actor
on a local lotus network.

#### Integration test framework

Create a new test in the `fvm_integration_tests` crate and use the path of the compiled wasm file to retrieve its binary content.
You will then be able to interact with it. An example of such tests can be found in `<ref-fvm-repository>/testing/integation/tests`.

#### Lotus local network

1. Setup a Lotus devnet on the branch `experimental/fvm-m2`. Instructions can be found 
[in the documentation](https://lotus.filecoin.io/developers/local-network/).
2. Install the actor on the lotus devnet, `lotus chain install-actor <path-to-wasm-bytecode>`. This command should return
the CID representing the bytecode of the actor.
3. Instantiate the actor, `lotus chain create-actor <code-cid> <encoded-params>`. This command should return the address
at which the actor is instantiated.
4. Invoke any function from your actor, `lotus chain invoke <address> <method-num>`

## License

Dual-licensed: [MIT](./LICENSE-MIT), [Apache Software License v2](./LICENSE-APACHE), by way of the
[Permissive License Stack](https://protocol.ai/blog/announcing-the-permissive-license-stack/).
