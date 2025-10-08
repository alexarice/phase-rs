//! Functions and datastructures for type checking

use std::collections::HashMap;

use miette::Diagnostic;
use thiserror::Error;

use crate::{
    raw_syntax::{PatternR, TermR, pattern::PatTensorR, term::TensorR},
    text::{Name, Span},
    typed_syntax::{PatternType, TermT, TermType},
};

/// Errors that can occur during typechecking.
#[derive(Error, Diagnostic, Debug, Clone)]
pub enum TypeCheckError<S: Span> {
    /// Error for mismatching type between terms in a composition.
    #[error("Type mismatch when composing")]
    #[diagnostic(code("Type mismatch."))]
    TypeMismatch {
        /// Term 1
        #[label("Has type {ty1}")]
        t1: TensorR<S>,
        /// Type of term 1
        ty1: TermType,
        /// Term 2
        #[label("Has type {ty2}")]
        t2: TensorR<S>,
        /// Type of term 2
        ty2: TermType,
    },
    /// Error for mismatching type between a term and pattern in an "if let" statement.
    #[error("Type mismatch between pattern and term in 'if let'")]
    #[diagnostic(code("If let type mismatch."))]
    IfTypeMismatch {
        /// Pattern
        #[label("Has type {pty}")]
        p: PatternR<S>,
        /// Type of pattern
        pty: PatternType,
        /// Body term
        #[label("Has type {tty}")]
        t: TensorR<S>,
        /// Type of body term
        tty: TermType,
    },
    /// Error for mismatching type between composed patterns.
    #[error("Type mismatch when composing patterns")]
    #[diagnostic(code("Pattern type mismatch."))]
    PatternTypeMismatch {
        /// Pattern 1
        #[label("Has type {ty1}")]
        p1: PatTensorR<S>,
        /// Type of pattern 1
        ty1: PatternType,
        /// Pattern 2
        #[label("Has type {ty2}")]
        p2: PatTensorR<S>,
        /// Type of pattern 2
        ty2: PatternType,
    },
    /// Error for an unknown top-level symbol.
    #[error("Unrecognised top-level symbol {name}.")]
    #[diagnostic(code("Unknown symbol."))]
    UnknownSymbol {
        /// The unknown symbol encountered
        name: Name,
        /// Span of symbol
        #[label("Symbol used here")]
        span: S,
    },
    /// Error for when a sqrt operation is applied to a term with compositions.
    #[error("Tried to root unrootable unitary term.")]
    #[diagnostic(code("Invalid root."))]
    TermNotRootable {
        /// Term which contains compositions
        tm: TermR<S>,
        #[label("This compostion prevents square rooting")]
        /// Span of sqrt term causing error
        #[label("Square root applied here")]
        span_of_root: S,
    },
}

/// Typing enviroment, holding definitions of top level symbols.
#[derive(Default)]
pub struct Env(pub(crate) HashMap<Name, TermT>);
