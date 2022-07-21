# Procedural macro tests

As most of our testing cases have to do with Rust compilation our tests have to be conducted at build time.

To be able to test those cases, a dedicated Rust crate is used, [_trybuild_](https://github.com/dtolnay/trybuild).

_trybuild_ is a crate that will first try to compile the files in the test folder and then generate files compiling
all outputs on _stderr_. Those files will later on serve as references and all later tests will verify that all _stderr_
messages outputs are the same as the references.

When running some new tests for the first time a `wip` folder is generated in the `macro` folder. It contains the
new references. Those references should be moved to their respective test folder to ensure future tests to be correct.