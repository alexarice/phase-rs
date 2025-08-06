//! Structure for representing phases, elements of the unit circle on the complex plane.

use pretty::RcDoc;
use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::{float, multispace0},
    combinator::{alt, delimited},
};

use crate::text::ToDoc;

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

    /// Returns the angle specified by this phase, divided by pi.
    /// e.g. if `phase.eval() == 1.0` then `phase` represents the angle `pi`
    pub fn eval(&self) -> f64 {
        match self {
            Phase::Angle(a) => *a,
            Phase::MinusOne => 1.0,
            Phase::Imag => 0.5,
            Phase::MinusImag => 1.5,
        }
    }
}

/// Parser for phases.
pub fn phase(input: &mut LocatingSlice<&str>) -> ModalResult<Phase> {
    alt((
        "-1".value(Phase::MinusOne),
        "i".value(Phase::Imag),
        "-i".value(Phase::MinusImag),
        delimited(
            ("ph(", multispace0),
            float,
            (multispace0, "pi", multispace0, ")"),
        )
        .map(Phase::Angle),
    ))
    .parse_next(input)
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
