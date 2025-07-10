//! The combinator version of the language.
//!
//! ## Example
//! To parse from string `src`, typecheck, evaluate, and print a command, the following can be run:
//! ```rust
//! let parsed = terminated(command, multispace0)
//!     .parse(LocatingSlice::new(src))
//!     .map_err(|e| anyhow::format_err!("{e}"))?;
//! let (_env, checked) = parsed.check().map_err(|e| anyhow::format_err!("{e:?}"))?;
//! let mut evalled: TermN = checked.eval();
//! evalled.squash();
//! let quoted = evalled.quote();
//! let raw = quoted.to_raw();
//! println!("Evaluated:\n{}\n", raw.to_doc().pretty(60));
//! ```

pub mod command;
pub mod eval;
pub mod eval_circ;
pub mod parsing;
pub mod syntax;
pub mod typecheck;
pub mod unitary;
