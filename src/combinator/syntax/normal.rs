//! Normal-form syntax definitions
//!
//! Syntax for evaluated terms.
//! This is assumed to be typechecked/well-formed.

use super::{
    KetState,
    typed::{PatternType, TermType},
};

/// A normal-form term
#[derive(Clone, Debug, PartialEq)]
pub enum TermN {
    /// A composition "t_1 ; ... ; t_n" with given type
    Comp(Vec<TermN>, TermType),
    /// A tensor "t_1 x ... x t_n"
    Tensor(Vec<TermN>),
    /// An "atomic" term
    Atom(AtomN),
}

/// "Atomic" terms. Terms which are not compositions or tensors.
#[derive(Clone, Debug, PartialEq)]
pub enum AtomN {
    /// A (global) phase operator, e.g. "-1" or "ph(0.1pi)"
    Phase(f64),
    /// An "if let" statement with given pattern, body term, and type
    IfLet(PatternN, Box<TermN>, TermType),
}

/// A normal-form patterns
#[derive(Clone, Debug, PartialEq)]
pub enum PatternN {
    /// A composition "p_1 . ... . p_n" with given type
    Comp(Vec<PatternN>, PatternType),
    /// A tensor "p_1 x ... x p_n"
    Tensor(Vec<PatternN>),
    /// A single ket state "|x>"
    Ket(KetState),
    /// An "atomic" term. Compound terms are evaluated to pattern compositions/tensors.
    Unitary(Box<AtomN>),
}

impl AtomN {
    pub(crate) fn get_type(&self) -> TermType {
        match self {
            AtomN::Phase(_) => TermType(0),
            AtomN::IfLet(_, _, ty) => *ty,
        }
    }
}
