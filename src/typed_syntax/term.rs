//! Term syntax terms.

use std::iter::Sum;

use crate::{
    circuit_syntax::{TermC, pattern::PatternC, term::ClauseC},
    normal_syntax::{Buildable, term::AtomN},
    phase::Phase,
    raw_syntax::{
        TermR,
        term::{AtomR, AtomRInner, TensorR, TensorRInner, TermRInner},
    },
    text::Name,
    typed_syntax::{PatternT, PatternType},
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

    /// Evaluate a term to a given `Buildable` type, expanding top level definitions
    /// and evaluating inverse and sqrt macros.
    /// In particular this can be used to generate a `TermN` from a `TermT`.
    pub fn eval<B: Buildable>(&self) -> B {
        self.eval_with_phase_mul(1.0)
    }

    pub(super) fn eval_with_phase_mul<B: Buildable>(&self, phase_mul: f64) -> B {
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

    pub(super) fn eval_circ_clause(
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
