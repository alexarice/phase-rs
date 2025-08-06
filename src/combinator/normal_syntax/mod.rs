//! Normal-form syntax definitions
//!
//! Syntax for evaluated terms.
//! This is assumed to be typechecked/well-formed.

use std::f64::consts::PI;

use faer::{Mat, mat};
use num_complex::Complex;

use super::typed_syntax::{PatternT, PatternType, TermT, TermType};
use crate::{ket::KetState, phase::Phase};

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
}

impl TermN {
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
    fn quote(&self) -> TermT {
        match self {
            AtomN::Phase(angle) => TermT::Phase(Phase::from_angle(*angle)),
            AtomN::IfLet(pattern, inner, _) => TermT::IfLet {
                pattern: pattern.quote(),
                inner: Box::new(inner.quote()),
            },
        }
    }

    fn squash(&mut self) {
        if let AtomN::IfLet(pattern, inner, _) = self {
            pattern.squash();
            inner.squash();
        }
    }
}

impl PatternN {
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
            PatternN::Ket(state) => PatternT::Ket(vec![*state]),
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

/// Trait for objects that can built with compositions, tensors, or from an `AtomN`.
pub trait Buildable {
    /// Build a composition object from a sequence of subobjects and a given type.
    /// Subobjects should be given in diagrammatic order, not function composition order.
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self;
    /// Build a tensor product from a sequence of subobjects.
    fn tensor(iter: impl Iterator<Item = Self>) -> Self;
    /// Build an object from an atom.
    fn atom(atom: AtomN) -> Self;
}

impl Buildable for TermN {
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self {
        TermN::Comp(iter.collect(), *ty)
    }

    fn tensor(iter: impl Iterator<Item = Self>) -> Self {
        TermN::Tensor(iter.collect())
    }

    fn atom(atom: AtomN) -> Self {
        TermN::Atom(atom)
    }
}

impl Buildable for PatternN {
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self {
        PatternN::Comp(iter.rev().collect(), PatternType(ty.0, ty.0))
    }

    fn tensor(iter: impl Iterator<Item = Self>) -> Self {
        PatternN::Tensor(iter.collect())
    }

    fn atom(atom: AtomN) -> Self {
        PatternN::Unitary(Box::new(atom))
    }
}
