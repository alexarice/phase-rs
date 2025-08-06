//! Structure for representing primitive states in ket notation.

use std::f64::consts::FRAC_1_SQRT_2;

use faer::{Mat, mat};
use num_complex::Complex;
use pretty::RcDoc;
use winnow::{
    LocatingSlice, ModalResult, Parser,
    combinator::{alt, delimited, repeat},
};

use crate::text::ToDoc;

/// Holds the value of a ket pattern.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KetState {
    /// |0> pattern
    Zero,
    /// |1> pattern
    One,
    /// |+> pattern
    Plus,
    /// |-> pattern
    Minus,
}

impl KetState {
    /// Returns the complement of the state.
    /// `state` and `state.compl()` from a basis of C^2
    pub fn compl(self) -> Self {
        match self {
            KetState::Zero => KetState::One,
            KetState::One => KetState::Zero,
            KetState::Plus => KetState::Minus,
            KetState::Minus => KetState::Plus,
        }
    }

    /// Returns the character needed to print this ket state.
    pub fn to_char(&self) -> char {
        match self {
            KetState::Zero => '0',
            KetState::One => '1',
            KetState::Plus => '+',
            KetState::Minus => '-',
        }
    }
}

/// Parse a composite ket state of the form '|("0"|"1"|"+"|"-")+>'
pub fn ket(input: &mut LocatingSlice<&str>) -> ModalResult<Vec<KetState>> {
    delimited(
        "|",
        repeat(
            1..,
            alt((
                "0".value(KetState::Zero),
                "1".value(KetState::One),
                "+".value(KetState::Plus),
                "-".value(KetState::Minus),
            )),
        ),
        ">",
    )
    .parse_next(input)
}

impl ToDoc for Vec<KetState> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::text("|")
            .append(self.iter().map(KetState::to_char).collect::<String>())
            .append(">")
    }
}

const CISQRT2: Complex<f64> = Complex::new(FRAC_1_SQRT_2, 0.0);

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
