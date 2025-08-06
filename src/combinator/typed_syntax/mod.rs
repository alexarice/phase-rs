//! Typed syntax definitions
//!
//! The core syntax of the tool.
//! This is assumed to be typechecked/well-formed.

use std::iter::Sum;

use super::normal_syntax::{AtomN, Buildable, PatternN};
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

    /// Evaluate a term to a `PatternN`, expanding top level definitions
    /// and evaluating inverse and sqrt macros.
    fn eval(&self) -> PatternN {
        match self {
            PatternT::Comp(patterns) => {
                if patterns.len() == 1 {
                    patterns[0].eval()
                } else {
                    PatternN::Comp(
                        patterns.iter().map(PatternT::eval).collect(),
                        self.get_type(),
                    )
                }
            }
            PatternT::Tensor(patterns) => {
                if patterns.len() == 1 {
                    patterns[0].eval()
                } else {
                    PatternN::Tensor(patterns.iter().map(PatternT::eval).collect())
                }
            }
            PatternT::Ket(states) => {
                PatternN::Tensor(states.iter().map(|&state| PatternN::Ket(state)).collect())
            }
            PatternT::Unitary(inner) => inner.eval(),
        }
    }
}

impl TermT {
    /// Evaluate a term to a given `Buildable` type, expanding top level definitions
    /// and evaluating inverse and sqrt macros.
    /// In particular this can be used to generate a `TermN` from a `TermT`.
    pub fn eval<B: Buildable>(&self) -> B {
        self.eval_with_phase_mul(1.0)
    }

    fn eval_with_phase_mul<B: Buildable>(&self, phase_mul: f64) -> B {
        match self {
            TermT::Comp(terms) => {
                let mut mapped_terms = terms.iter().map(|t| t.eval_with_phase_mul(phase_mul));
                if terms.len() == 1 {
                    mapped_terms.next().unwrap()
                } else if phase_mul > 0.0 {
                    B::comp(mapped_terms, &terms.first().unwrap().get_type())
                } else {
                    B::comp(mapped_terms.rev(), &terms.first().unwrap().get_type())
                }
            }
            TermT::Tensor(terms) => {
                if terms.len() == 1 {
                    terms[0].eval_with_phase_mul(phase_mul)
                } else {
                    B::tensor(terms.iter().map(|t| t.eval_with_phase_mul(phase_mul)))
                }
            }
            TermT::Id(ty) => B::comp(std::iter::empty(), ty),
            TermT::Phase(phase) => B::atom(AtomN::Phase(phase_mul * phase.eval())),
            TermT::IfLet { pattern, inner } => B::atom(AtomN::IfLet(
                pattern.eval(),
                Box::new(inner.eval_with_phase_mul(phase_mul)),
                TermType(pattern.get_type().0),
            )),
            TermT::Gate { def, .. } => def.eval_with_phase_mul(phase_mul),
            TermT::Inverse(inner) => inner.eval_with_phase_mul(-phase_mul),
            TermT::Sqrt(inner) => inner.eval_with_phase_mul(phase_mul / 2.0),
        }
    }
}
