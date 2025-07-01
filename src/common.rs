use std::ops::Range;

use pretty::RcDoc;
use winnow::{LocatingSlice, Parser};

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<S, T> {
    pub inner: T,
    pub span: S,
}

pub trait ToDoc {
    fn to_doc(&self) -> RcDoc;
}

pub fn parse_spanned<'a, T, E>(
    inner: impl Parser<LocatingSlice<&'a str>, T, E>,
) -> impl Parser<LocatingSlice<&'a str>, Spanned<Range<usize>, T>, E> {
    inner
        .with_span()
        .map(|(t, span)| Spanned { inner: t, span })
}

impl<S, T: ToDoc> ToDoc for Spanned<S, T> {
    fn to_doc(&self) -> RcDoc {
        self.inner.to_doc()
    }
}

impl<T> From<T> for Spanned<(), T> {
    fn from(value: T) -> Self {
        Spanned {
            inner: value,
            span: (),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KetState {
    Zero,
    One,
    Plus,
    Minus,
}

impl KetState {
    pub fn compl(self) -> Self {
        match self {
            KetState::Zero => KetState::One,
            KetState::One => KetState::Zero,
            KetState::Plus => KetState::Minus,
            KetState::Minus => KetState::Plus,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            KetState::Zero => '0',
            KetState::One => '1',
            KetState::Plus => '+',
            KetState::Minus => '-',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Angle(f64),
    MinusOne,
    Imag,
    MinusImag,
}

impl Phase {
    pub fn from_angle(f: f64) -> Self {
        if f == 0.5 {
            Phase::Imag
        } else if f == 1.0 {
            Phase::MinusOne
        } else if f == 1.0 {
            Phase::MinusImag
        } else {
            Phase::Angle(f)
        }
    }
}

impl Phase {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            Phase::Angle(a) => RcDoc::text(format!("ph({a}pi)")),
            Phase::MinusOne => RcDoc::text("-1"),
            Phase::Imag => RcDoc::text("i"),
            Phase::MinusImag => RcDoc::text("-i"),
        }
    }
}
