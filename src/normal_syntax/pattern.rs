//! Normal form patterns

use faer::Mat;
use num_complex::Complex;

use crate::{
    ket::{CompKetState, KetState},
    normal_syntax::term::AtomN,
    typed_syntax::{PatternT, PatternType, TermT, TermType},
};

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

impl PatternN {
    /// Convert a normal-form pattern of type qm < qn to an m x n isometry matrix `i`
    /// and an n x n projector `p` such that
    /// p + ii^dagger = id
    pub fn to_inj_and_proj(&self) -> (Mat<Complex<f64>>, Mat<Complex<f64>>) {
        match self {
            PatternN::Comp(patterns, ty) => {
                let mut patterns_iter = patterns.iter().map(PatternN::to_inj_and_proj);
                if let Some(i) = patterns_iter.next() {
                    patterns_iter.fold(i, |(i1, p1), (i2, p2)| {
                        (&i1 * i2, p1 + &i1 * p2 * i1.adjoint())
                    })
                } else {
                    (
                        Mat::identity(1 << ty.0, 1 << ty.0),
                        Mat::zeros(1 << ty.0, 1 << ty.0),
                    )
                }
            }
            PatternN::Tensor(patterns) => {
                let mut patterns_iter = patterns.iter().map(PatternN::to_inj_and_proj);
                let i = patterns_iter.next().unwrap();
                patterns_iter.fold(i, |(i1, p1), (i2, p2)| {
                    (
                        i1.kron(i2),
                        p1.kron(Mat::<Complex<f64>>::identity(p2.nrows(), p2.nrows()))
                            + (&i1 * i1.adjoint()).kron(p2),
                    )
                })
            }
            PatternN::Ket(state) => {
                let m = state.to_state();
                let cm = state.compl().to_state();
                (m, cm.as_ref() * cm.adjoint())
            }
            PatternN::Unitary(inner) => {
                let size = inner.get_type().0;
                (inner.to_unitary(), Mat::zeros(1 << size, 1 << size))
            }
        }
    }

    /// Return a `PatternT` which is the "quotation" of this normal-form pattern.
    /// Realises that all normal-form patterns are also patterns.
    pub fn quote(&self) -> PatternT {
        match self {
            PatternN::Comp(patterns, ty) => {
                if patterns.is_empty() {
                    PatternT::Unitary(Box::new(TermT::Id(TermType(ty.0))))
                } else {
                    PatternT::Comp(patterns.iter().map(PatternN::quote).collect())
                }
            }
            PatternN::Tensor(patterns) => {
                PatternT::Tensor(patterns.iter().map(PatternN::quote).collect())
            }
            PatternN::Ket(state) => PatternT::Ket(CompKetState::single(*state)),
            PatternN::Unitary(inner) => PatternT::Unitary(Box::new(inner.quote())),
        }
    }

    fn squash_comp(mut self, acc: &mut Vec<PatternN>) {
        if let PatternN::Comp(patterns, _) = self {
            for p in patterns {
                p.squash_comp(acc);
            }
        } else {
            self.squash();
            acc.push(self);
        }
    }

    fn squash_tensor(mut self, acc: &mut Vec<PatternN>) {
        if let PatternN::Tensor(patterns) = self {
            for p in patterns {
                p.squash_tensor(acc);
            }
        } else {
            self.squash();
            acc.push(self);
        }
    }

    /// Simplifies compositions, tensors, and identities in the given normal-form pattern.
    pub fn squash(&mut self) {
        match self {
            PatternN::Comp(patterns, _) => {
                let old_patterns = std::mem::take(patterns);
                for p in old_patterns {
                    p.squash_comp(patterns);
                }
                if patterns.len() == 1 {
                    *self = patterns.pop().unwrap();
                }
            }
            PatternN::Tensor(patterns) => {
                let old_patterns = std::mem::take(patterns);
                for p in old_patterns {
                    p.squash_tensor(patterns);
                }
                if patterns.len() == 1 {
                    *self = patterns.pop().unwrap();
                }
            }
            PatternN::Ket(_) => {}
            PatternN::Unitary(inner) => inner.squash(),
        }
    }
}
