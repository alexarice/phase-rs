//! Contains functions for converting normal-form syntax to matrices.

use std::f64::consts::{FRAC_1_SQRT_2, PI};

use faer::{Mat, mat};
use num_complex::Complex;

const CISQRT2: Complex<f64> = Complex::new(FRAC_1_SQRT_2, 0.0);

use super::syntax::{
    KetState,
    normal::{AtomN, PatternN, TermN},
};

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

impl KetState {
    /// Returns the vector this `KetState` represents.
    pub fn to_state(self) -> Mat<Complex<f64>> {
        match self {
            KetState::Zero => mat![[Complex::ONE], [Complex::ZERO]],
            KetState::One => mat![[Complex::ZERO], [Complex::ONE]],
            KetState::Plus => mat![[CISQRT2], [CISQRT2]],
            KetState::Minus => mat![[CISQRT2], [-CISQRT2]],
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
