//! The combinator version of the language.
//!
//! ## Example
//! To parse from string `src`, typecheck, evaluate, and print a command, the following can be run:
//! ```rust
//! use phase_rs::combinator::{
//!    parsing::command,
//!    syntax::{ToDoc, normal::TermN},
//! };
//! use winnow::{LocatingSlice, Parser, ascii::multispace0, combinator::terminated};
//! fn parse_and_eval(src: &str) {
//!   let parsed = terminated(command, multispace0)
//!       .parse(LocatingSlice::new(src)).unwrap();
//!   let (_env, checked) = parsed.check().unwrap();
//!   let mut evalled: TermN = checked.eval();
//!   evalled.squash();
//!   let quoted = evalled.quote();
//!   let raw = quoted.to_raw();
//!   println!("Evaluated:\n{}\n", raw.to_doc().pretty(60));
//! }
//! ```

pub mod circuit_syntax;
pub mod command;
pub mod normal_syntax;
pub mod raw_syntax;
pub mod typecheck;
pub mod typed_syntax;
