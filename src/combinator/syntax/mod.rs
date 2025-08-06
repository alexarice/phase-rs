//! Syntax definitions for raw, typed, normal, and circuit-normal syntax.

use pretty::RcDoc;

use crate::text::ToDoc;

pub mod circuit_normal;
pub mod normal;
pub mod raw;
pub mod typed;

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

/// Represents a (global) phase operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Specifies the phase by an float, which should equal the specified angle divided by pi
    Angle(f64),
    /// -1 phase, equivalent to `Angle(1.0)`
    MinusOne,
    /// i phase, equivalent to `Angle(0.5)`
    Imag,
    /// -i phase, equivalent to `Angle(1.5)`
    MinusImag,
}

impl Phase {
    /// Construct a new `Phase` from a float representing the desired angle divided by pi.
    /// Uses special phase enum variants when possible.
    pub fn from_angle(f: f64) -> Self {
        if f == 0.5 {
            Phase::Imag
        } else if f == 1.0 {
            Phase::MinusOne
        } else if f == 1.5 {
            Phase::MinusImag
        } else {
            Phase::Angle(f)
        }
    }
}

impl ToDoc for Phase {
    fn to_doc(&self) -> RcDoc {
        match self {
            Phase::Angle(a) => RcDoc::text(format!("ph({a}pi)")),
            Phase::MinusOne => RcDoc::text("-1"),
            Phase::Imag => RcDoc::text("i"),
            Phase::MinusImag => RcDoc::text("-i"),
        }
    }
}
