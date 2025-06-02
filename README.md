# Prototype interpreter for "it's just a phase"

The repository contains an implementation of the combinator variant of the "it's just a phase" language.
Given an input program, the interpreter:
- performs some basic normalisation (assocatiativity etc.)
- performs macro expansion of inverses, square roots, and gate definitions
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

## Examples

Examples of common gates (and of the syntax) can be found in [examples/gates.ph](examples/gates.ph).
