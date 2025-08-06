//! Raw syntax definitions
//!
//! Raw syntax is used primarily for parsing and printed.
//! It is not assumed to be typechecked/well-formed.

pub mod term;
pub use term::TermR;

pub mod pattern;
pub use pattern::PatternR;
