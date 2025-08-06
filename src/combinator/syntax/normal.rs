//! Normal-form syntax definitions
//!
//! Syntax for evaluated terms.
//! This is assumed to be typechecked/well-formed.

use super::typed::{PatternType, TermType};
use crate::ket::KetState;

use std::f64::consts::PI;

use faer::{Mat, mat};
use num_complex::Complex;



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
