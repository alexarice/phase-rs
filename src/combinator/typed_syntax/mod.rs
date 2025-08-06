//! Typed syntax definitions
//!
//! The core syntax of the tool.
//! This is assumed to be typechecked/well-formed.

use std::iter::Sum;

use super::normal_syntax::{AtomN, Buildable, PatternN};
use crate::{
    combinator::{
        circuit_syntax::{ClauseC, PatternC, TermC},
        raw_syntax::{
            pattern::{PatAtomR, PatAtomRInner, PatTensorR, PatTensorRInner, PatternRInner}, term::{AtomR, AtomRInner, TensorR, TensorRInner, TermRInner}, PatternR, TermR
        },
    },
    ket::CompKetState,
    phase::Phase, text::Name,
};

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
        name: Name,
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
    Ket(CompKetState),
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
            PatternT::Ket(states) => PatternType(states.qubits(), 0),
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

    fn eval_circ(&self, pattern: &mut PatternC, inj: &mut Vec<usize>, clauses: &mut Vec<ClauseC>) {
        match self {
            PatternT::Comp(patterns) => {
                for p in patterns {
                    p.eval_circ(pattern, inj, clauses);
                }
            }
            PatternT::Tensor(patterns) => {
                let mut stack: Vec<Vec<usize>> = Vec::new();
                for p in patterns.iter().rev() {
                    let size = p.get_type().0;
                    let mut i = inj.split_off(inj.len() - size);
                    p.eval_circ(pattern, &mut i, clauses);
                    stack.push(i);
                }
                while let Some(i) = stack.pop() {
                    inj.extend(i);
                }
            }
            PatternT::Ket(states) => {
                for (state, i) in states.iter().zip(inj.drain(0..states.qubits())) {
                    pattern.parts[i] = Some(*state)
                }
            }
            PatternT::Unitary(inner) => {
                inner.eval_circ_clause(pattern, inj, -1.0, clauses);
            }
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

    /// Returns a `TermC` representing the "circuit-normal-form" of the term.
    pub fn eval_circ(&self) -> TermC {
        let mut clauses = vec![];
        let size = self.get_type().0;
        let inj = (0..size).collect::<Vec<_>>();
        self.eval_circ_clause(&PatternC::id(size), &inj, 1.0, &mut clauses);
        TermC {
            clauses,
            ty: self.get_type(),
        }
    }
    fn eval_circ_clause(
        &self,
        pattern: &PatternC,
        inj: &[usize],
        phase_mul: f64,
        clauses: &mut Vec<ClauseC>,
    ) {
        match self {
            TermT::Comp(terms) => {
                if phase_mul < 0.0 {
                    for t in terms.iter().rev() {
                        t.eval_circ_clause(pattern, inj, phase_mul, clauses);
                    }
                } else {
                    for t in terms {
                        t.eval_circ_clause(pattern, inj, phase_mul, clauses);
                    }
                }
            }
            TermT::Tensor(terms) => {
                let mut start = 0;
                for t in terms {
                    let size = t.get_type().0;
                    let end = start + size;
                    t.eval_circ_clause(pattern, &inj[start..end], phase_mul, clauses);
                    start = end;
                }
            }
            TermT::Id(_) => {
                // Intentionally blank
            }
            TermT::Phase(phase) => {
                clauses.push(ClauseC {
                    pattern: pattern.clone(),
                    phase: phase_mul * phase.eval(),
                });
            }
            TermT::IfLet {
                pattern: if_pattern,
                inner,
            } => {
                let mut unitary_clauses = Vec::new();
                let mut inner_pattern = pattern.clone();
                let mut inner_inj = inj.to_vec();
                if_pattern.eval_circ(&mut inner_pattern, &mut inner_inj, &mut unitary_clauses);
                let temp: Vec<_> = unitary_clauses.iter().rev().map(ClauseC::invert).collect();
                clauses.extend(unitary_clauses);

                inner.eval_circ_clause(&inner_pattern, &inner_inj, phase_mul, clauses);

                clauses.extend(temp)
            }
            TermT::Gate { def, .. } => {
                def.eval_circ_clause(pattern, inj, phase_mul, clauses);
            }
            TermT::Inverse(inner) => {
                inner.eval_circ_clause(pattern, inj, -phase_mul, clauses);
            }
            TermT::Sqrt(inner) => {
                inner.eval_circ_clause(pattern, inj, phase_mul / 2.0, clauses);
            }
        }
    }
}

impl TermT {
    /// Convert to a raw term.
    pub fn to_raw(&self) -> TermR<()> {
        let terms = if let TermT::Comp(terms) = self {
            terms.iter().map(|t| t.to_raw_tensor()).collect()
        } else {
            vec![self.to_raw_tensor()]
        };
        TermRInner { terms }.into()
    }

    fn to_raw_tensor(&self) -> TensorR<()> {
        let terms = if let TermT::Tensor(terms) = self {
            terms.iter().map(|t| t.to_raw_atom()).collect()
        } else {
            vec![self.to_raw_atom()]
        };
        TensorRInner { terms }.into()
    }

    fn to_raw_atom(&self) -> AtomR<()> {
        match self {
            TermT::Id(ty) => AtomRInner::Id(ty.0),
            TermT::Phase(phase) => AtomRInner::Phase(*phase),
            TermT::IfLet { pattern, inner } => AtomRInner::IfLet {
                pattern: pattern.to_raw(),
                inner: Box::new(inner.to_raw_atom()),
            },
            TermT::Gate { name, .. } => AtomRInner::Gate(name.to_owned()),
            TermT::Inverse(inner) => AtomRInner::Inverse(Box::new(inner.to_raw_atom())),
            TermT::Sqrt(inner) => AtomRInner::Sqrt(Box::new(inner.to_raw_atom())),
            t => AtomRInner::Brackets(t.to_raw()),
        }
        .into()
    }
}

impl PatternT {
    /// Convert to a raw pattern.
    pub fn to_raw(&self) -> PatternR<()> {
        let patterns = if let PatternT::Comp(patterns) = self {
            patterns.iter().map(|t| t.to_raw_tensor()).collect()
        } else {
            vec![self.to_raw_tensor()]
        };
        PatternRInner { patterns }.into()
    }

    fn to_raw_tensor(&self) -> PatTensorR<()> {
        let patterns = if let PatternT::Tensor(patterns) = self {
            patterns.iter().map(|t| t.to_raw_atom()).collect()
        } else {
            vec![self.to_raw_atom()]
        };
        PatTensorRInner { patterns }.into()
    }

    fn to_raw_atom(&self) -> PatAtomR<()> {
        match self {
            PatternT::Ket(states) => PatAtomRInner::Ket(states.clone()),
            PatternT::Unitary(inner) => PatAtomRInner::Unitary(Box::new(inner.to_raw())),
            p => PatAtomRInner::Brackets(p.to_raw()),
        }
        .into()
    }
}
