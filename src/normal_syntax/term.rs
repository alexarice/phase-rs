//! Normal-form terms.

use std::f64::consts::PI;

use faer::{Mat, mat};
use num_complex::Complex;

use crate::{
    normal_syntax::PatternN,
    phase::Phase,
    typed_syntax::{TermT, TermType},
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

impl TermN {
    /// Convert a normal-form term of type qn <-> qn to an n x n unitary matrix.
    pub fn to_unitary(&self) -> Mat<Complex<f64>> {
        match self {
            TermN::Comp(terms, ty) => {
                let mut terms_iter = terms.iter().map(TermN::to_unitary);
                match terms_iter.next() {
                    None => Mat::identity(1 << ty.0, 1 << ty.0),
                    Some(u) => terms_iter.fold(u, |x, y| y * x),
                }
            }
            TermN::Tensor(terms) => {
                let mut terms_iter = terms.iter().map(TermN::to_unitary);
                match terms_iter.next() {
                    None => Mat::identity(1, 1),
                    Some(u) => terms_iter.fold(u, |x, y| x.kron(y)),
                }
            }
            TermN::Atom(atom) => atom.to_unitary(),
        }
    }

    /// Return a `TermT` which is the "quotation" of this normal-form term.
    /// Realises that all normal-form terms are also terms.
    pub fn quote(&self) -> TermT {
        match self {
            TermN::Comp(terms, ty) => {
                if terms.is_empty() {
                    TermT::Id(*ty)
                } else {
                    TermT::Comp(terms.iter().map(TermN::quote).collect())
                }
            }
            TermN::Tensor(terms) => TermT::Tensor(terms.iter().map(TermN::quote).collect()),
            TermN::Atom(atom) => atom.quote(),
        }
    }

    fn squash_comp(mut self, acc: &mut Vec<TermN>) {
        if let TermN::Comp(terms, _) = self {
            for t in terms {
                t.squash_comp(acc);
            }
        } else {
            self.squash();
            acc.push(self);
        }
    }

    fn squash_tensor(mut self, acc: &mut Vec<TermN>) {
        if let TermN::Tensor(terms) = self {
            for t in terms {
                t.squash_tensor(acc);
            }
        } else {
            self.squash();
            acc.push(self);
        }
    }

    /// Simplifies compositions, tensors, and identities in the given normal-form term.
    pub fn squash(&mut self) {
        match self {
            TermN::Comp(terms, _) => {
                let old_terms = std::mem::take(terms);
                for t in old_terms {
                    t.squash_comp(terms);
                }
                if terms.len() == 1 {
                    *self = terms.pop().unwrap();
                }
            }
            TermN::Tensor(terms) => {
                let old_terms = std::mem::take(terms);
                for t in old_terms {
                    t.squash_tensor(terms);
                }
                if terms.len() == 1 {
                    *self = terms.pop().unwrap();
                }
            }
            TermN::Atom(atom) => atom.squash(),
        }
    }
}

impl AtomN {
    pub(crate) fn get_type(&self) -> TermType {
        match self {
            AtomN::Phase(_) => TermType(0),
            AtomN::IfLet(_, _, ty) => *ty,
        }
    }

    /// Convert a normal-form atom of type qn <-> qn to an n x n unitary matrix.
    pub fn to_unitary(&self) -> Mat<Complex<f64>> {
        match self {
            AtomN::Phase(angle) => mat![[Complex::cis(angle * PI)]],
            AtomN::IfLet(pattern, inner, _) => {
                let (inj, proj) = pattern.to_inj_and_proj();
                let u = inner.to_unitary();
                proj + &inj * u * inj.adjoint()
            }
        }
    }

    pub(super) fn quote(&self) -> TermT {
        match self {
            AtomN::Phase(angle) => TermT::Phase(Phase::from_angle(*angle)),
            AtomN::IfLet(pattern, inner, _) => TermT::IfLet {
                pattern: pattern.quote(),
                inner: Box::new(inner.quote()),
            },
        }
    }

    pub(super) fn squash(&mut self) {
        if let AtomN::IfLet(pattern, inner, _) = self {
            pattern.squash();
            inner.squash();
        }
    }
}
