//! Term syntax patterns.

use std::iter::Sum;

use crate::{
    circuit_syntax::{pattern::PatternC, term::ClauseC},
    ket::CompKetState,
    normal_syntax::PatternN,
    raw_syntax::{
        PatternR,
        pattern::{PatAtomR, PatAtomRInner, PatTensorR, PatTensorRInner, PatternRInner},
    },
    typed_syntax::TermT,
};

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
    pub(super) fn eval(&self) -> PatternN {
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

    pub(super) fn eval_circ(
        &self,
        pattern: &mut PatternC,
        inj: &mut Vec<usize>,
        clauses: &mut Vec<ClauseC>,
    ) {
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
