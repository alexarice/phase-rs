# Prototype interpreter for "it's just a phase"

The repository contains an implementation of the combinator variant of the "it's just a phase" language.
Given an input program, the interpreter:
- performs some basic normalisation (associativity etc.)
- performs macro expansion of inverses, square roots, and gate definitions
- compiles the term to a circuit definition.
- Builds the unitary for the output.

## Running

A file of commands can be run using:

```bash
cargo run -- --file <FILENAME>
```

or passed in through stdin. For all options see:
```bash
cargo run -- --help
```

Code documentation can be built by running:
```bash
cargo doc
```
and found (after building) at [target/doc/phase_rs/index.html](target/doc/phase_rs/index.html).

## Examples

Examples of common gates (and of the syntax) can be found in [examples/gates.ph](examples/gates.ph).
