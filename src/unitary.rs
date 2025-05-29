use faer::{Mat, mat};
use num_complex::Complex;
use std::f64::consts::{FRAC_1_SQRT_2, PI};

const CISQRT2: Complex<f64> = Complex::new(FRAC_1_SQRT_2, 0.0);

use crate::syntax::{
    KetState,
    typed::{PatternT, TermT},
};

impl TermT {
    pub fn to_unitary(&self) -> Mat<Complex<f64>> {
        match self {
            TermT::Comp { terms, ty } => {
                let mut terms_iter = terms.iter().map(TermT::to_unitary);
                match terms_iter.next() {
                    None => Mat::identity(1 << ty, 1 << ty),
                    Some(u) => terms_iter.fold(u, |x, y| y * x),
                }
            }
            TermT::Tensor { terms } => {
                let mut terms_iter = terms.iter().map(TermT::to_unitary);
                match terms_iter.next() {
                    None => Mat::identity(1, 1),
                    Some(u) => terms_iter.fold(u, |x, y| x.kron(y)),
                }
            }
            TermT::Phase { angle } => mat![[Complex::cis(angle * PI)]],
            TermT::IfLet { pattern, inner } => {
                let (inj, proj) = pattern.to_inj_and_proj();
                let u = inner.to_unitary();
                proj + inj.as_ref() * u * inj.adjoint()
            }
            TermT::Hadamard => {
                mat![[CISQRT2, CISQRT2], [CISQRT2, -CISQRT2]]
            }
            TermT::Gate { def, .. } => def.to_unitary(),
        }
    }
}

impl KetState {
    pub fn to_state(self) -> Mat<Complex<f64>> {
        match self {
            KetState::Zero => mat![[Complex::ONE], [Complex::ZERO]],
            KetState::One => mat![[Complex::ZERO], [Complex::ONE]],
            KetState::Plus => mat![[CISQRT2], [CISQRT2]],
            KetState::Minus => mat![[CISQRT2], [-CISQRT2]],
        }
    }
}

impl PatternT {
    pub fn to_inj_and_proj(&self) -> (Mat<Complex<f64>>, Mat<Complex<f64>>) {
        match self {
            PatternT::Comp { patterns } => {
                let mut patterns_iter = patterns.iter().map(PatternT::to_inj_and_proj);
                let i = patterns_iter.next().unwrap();
                patterns_iter.fold(i, |(i1, p1), (i2, p2)| {
                    (&i1 * i2, p1 + &i1 * p2 * i1.adjoint())
                })
            }
            PatternT::Tensor { patterns } => {
                let mut patterns_iter = patterns.iter().map(PatternT::to_inj_and_proj);
                let i = patterns_iter.next().unwrap();
                patterns_iter.fold(i, |(i1, p1), (i2, p2)| {
                    (
                        i1.kron(i2),
                        p1.kron(Mat::<Complex<f64>>::identity(p2.nrows(), p2.nrows()))
                            + (&i1 * i1.adjoint()).kron(p2),
                    )
                })
            }
            PatternT::Ket { state } => {
                let m = state.to_state();
                let cm = state.compl().to_state();
                (m, cm.as_ref() * cm.adjoint())
            }
            PatternT::Unitary(term_t) => {
                let size = term_t.get_type().0;
                (term_t.to_unitary(), Mat::zeros(1 << size, 1 << size))
            }
        }
    }
}
