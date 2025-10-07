//! The repository contains an implementation of the combinator variant of the "it's just a phase" language.
//! Given an input program, the interpreter:
//! - performs some basic normalisation (associativity etc.)
//! - performs macro expansion of inverses, square roots, and gate definitions
//! - compiles the term to a circuit definition.
//! - Builds the unitary for the output.
//!
//! ## Running
//!
//! A file of commands can be run using:
//!
//! ```bash
//! cargo run -- --file <FILENAME>
//! ```
//!
//! or passed in through stdin. For all options see:
//! ```bash
//! cargo run -- --help
//! ```

// #![warn(missing_docs)]

pub mod circuit_syntax;
pub mod command;
pub mod ket;
pub mod normal_syntax;
pub mod phase;
pub mod raw_syntax;
pub mod text;
pub mod typecheck;
pub mod typed_syntax;
