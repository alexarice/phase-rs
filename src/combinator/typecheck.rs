//! Functions and datastructures for type checking

use std::collections::HashMap;

use super::{
    raw_syntax::{
        AtomR, PatTensorR, PatternR, TensorR, TermR,
    },
    typed_syntax::{PatternType, TermT, TermType},
};

/// Errors that can occur during typechecking.
#[derive(Debug, Clone)]
pub enum TypeCheckError<S> {
    /// Error for mismatching type between terms in a composition.
    TypeMismatch {
        /// Term 1
        t1: TensorR<S>,
        /// Type of term 1
        ty1: TermType,
        /// Term 2
        t2: TensorR<S>,
        /// Type of term 2
        ty2: TermType,
    },
    /// Error for mismatching type between a term and pattern in an "if let" statement.
    IfTypeMismatch {
        /// Pattern
        p: PatternR<S>,
        /// Type of pattern
        pty: PatternType,
        /// Body term
        t: AtomR<S>,
        /// Type of body term
        tty: TermType,
    },
    /// Error for mismatching type between composed patterns.
    PatternTypeMismatch {
        /// Pattern 1
        p1: PatTensorR<S>,
        /// Type of pattern 1
        ty1: PatternType,
        /// Pattern 2
        p2: PatTensorR<S>,
        /// Type of pattern 2
        ty2: PatternType,
    },
    /// Error for an unknown top-level symbol.
    UnknownSymbol {
        /// The unknown symbol encountered
        name: String,
        /// Span of symbol
        span: S,
    },
    /// Error for when a sqrt operation is applied to a term with compositions.
    TermNotRootable {
        /// Term which contains compositions
        tm: TermR<S>,
        /// Span of sqrt term causing error
        span_of_root: S,
    },
}

/// Typing enviroment, holding definitions of top level symbols.
#[derive(Default)]
pub struct Env(pub(crate) HashMap<String, TermT>);
