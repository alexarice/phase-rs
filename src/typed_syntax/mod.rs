//! Typed syntax definitions
//!
//! The core syntax of the tool.
//! This is assumed to be typechecked/well-formed.

pub mod term;
pub use term::{TermT, TermType};

pub mod pattern;
pub use pattern::{PatternT, PatternType};
