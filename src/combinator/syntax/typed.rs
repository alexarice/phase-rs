//! Typed syntax definitions
//!
//! The core syntax of the tool.
//! This is assumed to be typechecked/well-formed.

use std::iter::Sum;

use crate::{ket::KetState, phase::Phase};

/// A unitary type "qn <-> qn"
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TermType(pub usize);

impl Sum for TermType {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        TermType(iter.map(|x| x.0).sum())
    }
}

impl TermType {
    /// Convert a unitary type qn <-> qn to pattern type qn < qn
    pub fn to_pattern_type(self) -> PatternType {
        PatternType(self.0, self.0)
    }
}

/// Syntax of typed terms
#[derive(Clone, Debug, PartialEq)]
pub enum TermT {
    /// A non-empty composition "t_1 ; ... ; t_n"
    Comp(Vec<TermT>),
    /// A tensor "t_1 x ... x t_n"
    Tensor(Vec<TermT>),
    /// An identity "id(n)"
    Id(TermType),
    /// A (global) phase operator, e.g. "-1" or "ph(0.1pi)"
    Phase(Phase),
    /// An "if let" statement, "if let pattern then inner"
    IfLet {
        /// Pattern to match on in "if let"
        pattern: PatternT,
        /// Body of the "if let"
        inner: Box<TermT>,
    },
    /// Top level symbol, a named gate
    Gate {
        /// Name of symbol/gate
        name: String,
        /// Definition of symbol
        def: Box<TermT>,
    },
    /// Inverse of a term "t ^ -1"
    Inverse(Box<TermT>),
    /// Square root of a term "sqrt(t)"
    Sqrt(Box<TermT>),
}

impl TermT {
    /// Returns the type of this term
    pub fn get_type(&self) -> TermType {
        match self {
            TermT::Comp(terms) => terms.first().unwrap().get_type(),
            TermT::Tensor(terms) => terms.iter().map(TermT::get_type).sum(),
            TermT::Id(ty) => *ty,
            TermT::Phase(_) => TermType(0),
            TermT::IfLet { pattern, .. } => TermType(pattern.get_type().0),
            TermT::Gate { def, .. } => def.get_type(),
            TermT::Inverse(inner) => inner.get_type(),
            TermT::Sqrt(inner) => inner.get_type(),
        }
    }
}

/// A pattern type "qn < qm"
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatternType(pub usize, pub usize);

impl Sum for PatternType {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(PatternType(0, 0), |PatternType(a, b), PatternType(c, d)| {
            PatternType(a + c, b + d)
        })
    }
}

/// Syntax of typed patterns
#[derive(Clone, Debug, PartialEq)]
pub enum PatternT {
    /// A non-empty composition "p_1 . ... . p_n"
    Comp(Vec<PatternT>),
    /// A tensor "p_1 x ... x p_n"
    Tensor(Vec<PatternT>),
    /// A sequence of ket states "|xyz>", equivalent to "|x> x |y> x |z>"
    Ket(Vec<KetState>),
    /// A unitary pattern
    Unitary(Box<TermT>),
}

impl PatternT {
    /// Returns the type of this pattern
    pub fn get_type(&self) -> PatternType {
        match self {
            PatternT::Comp(patterns) => PatternType(
                patterns.first().unwrap().get_type().0,
                patterns.last().unwrap().get_type().1,
            ),
            PatternT::Tensor(patterns) => patterns.iter().map(PatternT::get_type).sum(),
            PatternT::Ket(states) => PatternType(states.len(), 0),
            PatternT::Unitary(inner) => inner.get_type().to_pattern_type(),
        }
    }
}
