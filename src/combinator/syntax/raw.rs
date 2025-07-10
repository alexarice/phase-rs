//! Raw syntax definitions
//!
//! Raw syntax is used primarily for parsing and printed.
//! It is not assumed to be typechecked/well-formed.

use std::{borrow::Cow, ops::Range};

use pretty::RcDoc;
use winnow::{LocatingSlice, Parser};

use super::{KetState, Phase};
use crate::combinator::syntax::ToDoc;

/// Wraps data of type `T` in a span of type `S`, locating it in the source text.
/// The span is ignored when printing.
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<S, T> {
    /// Wrapped data
    pub inner: T,
    /// Text span
    pub span: S,
}

/// Parse a `Spanned<Range<usize>, T>` using a parser for `T`.
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

/// Raw syntax term with text span.
/// Represents a list of tensored terms composed together.
pub type TermR<S> = Spanned<S, TermRInner<S>>;

/// Raw syntax term without text span.
/// Represents a list of tensored terms composed together.
#[derive(Clone, Debug, PartialEq)]
pub struct TermRInner<S> {
    pub(crate) terms: Vec<TensorR<S>>,
}

/// Raw syntax tensored term with text span.
/// Represents a list of atoms tensored together.
pub type TensorR<S> = Spanned<S, TensorRInner<S>>;

/// Raw syntax tensored term without text span.
/// Represents a list of atoms tensored together.
#[derive(Clone, Debug, PartialEq)]
pub struct TensorRInner<S> {
    pub(crate) terms: Vec<AtomR<S>>,
}

/// Raw syntax atom with text span.
/// Represents a term other than a tensor or composition (or a composition/tensor in brackets)
pub type AtomR<S> = Spanned<S, AtomRInner<S>>;

/// Raw syntax atom without text span.
/// Represents a term other than a tensor or composition (or a composition/tensor in brackets)
#[derive(Clone, Debug, PartialEq)]
pub enum AtomRInner<S> {
    /// A term enclosed in parentheses
    Brackets(TermR<S>),
    /// An identity term "id(n)"
    Id(usize),
    /// A (global) phase operator, e.g. "-1" or "ph(0.1pi)"
    Phase(Phase),
    /// An "if let" statement, "if let pattern then inner"
    IfLet {
        /// Pattern to match on in "if let"
        pattern: PatternR<S>,
        /// Body of the "if let"
        inner: Box<AtomR<S>>,
    },
    /// Top level symbol, a named gate
    Gate(String),
    /// Inverse of a term "t ^ -1"
    Inverse(Box<AtomR<S>>),
    /// Square root of a term "sqrt(t)"
    Sqrt(Box<AtomR<S>>),
}

/// Raw syntax pattern with text span.
/// Represents a list of tensored patterns composed together.
pub type PatternR<S> = Spanned<S, PatternRInner<S>>;

/// Raw syntax pattern without text span.
/// Represents a list of tensored patterns composed together.
#[derive(Clone, Debug, PartialEq)]
pub struct PatternRInner<S> {
    pub(crate) patterns: Vec<PatTensorR<S>>,
}

/// Raw syntax tensored pattern with text span.
/// Represents a list of pattern atoms tensored together.
pub type PatTensorR<S> = Spanned<S, PatTensorRInner<S>>;

/// Raw syntax tensored pattern without text span.
/// Represents a list of pattern atoms tensored together.
#[derive(Clone, Debug, PartialEq)]
pub struct PatTensorRInner<S> {
    pub(crate) patterns: Vec<PatAtomR<S>>,
}

/// Raw syntax pattern atom with text span.
/// Represents a pattern other than a tensor or composition (or a composition/tensor in brackets)
pub type PatAtomR<S> = Spanned<S, PatAtomRInner<S>>;

/// Raw syntax pattern atom without text span.
/// Represents a pattern other than a tensor or composition (or a composition/tensor in brackets)
#[derive(Clone, Debug, PartialEq)]
pub enum PatAtomRInner<S> {
    /// A pattern enclosed in parentheses
    Brackets(PatternR<S>),
    /// A sequence of ket states "|xyz>", equivalent to "|x> x |y> x |z>"
    Ket(Vec<KetState>),
    /// A unitary pattern
    Unitary(Box<TermR<S>>),
}

impl<S> ToDoc for TermRInner<S> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.terms.iter().map(TensorR::to_doc),
            RcDoc::text(";").append(RcDoc::line()),
        )
        .group()
    }
}

impl<S> ToDoc for TensorRInner<S> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.terms.iter().map(AtomR::to_doc),
            RcDoc::line().append("x "),
        )
        .group()
    }
}

impl<S> ToDoc for AtomRInner<S> {
    fn to_doc(&self) -> RcDoc {
        match self {
            AtomRInner::Brackets(term) => RcDoc::text("(")
                .append(RcDoc::line().append(term.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
            AtomRInner::Id(qubits) => RcDoc::text(if *qubits == 1 {
                Cow::Borrowed("id")
            } else {
                Cow::Owned(format!("id{qubits}"))
            }),
            AtomRInner::Phase(phase) => phase.to_doc(),
            AtomRInner::IfLet { pattern, inner, .. } => RcDoc::text("if let")
                .append(RcDoc::line().append(pattern.to_doc()).nest(2))
                .append(RcDoc::line())
                .append("then")
                .group()
                .append(RcDoc::line().append(inner.to_doc()).nest(2))
                .group(),
            AtomRInner::Gate(name) => RcDoc::text(name),
            AtomRInner::Inverse(inner) => inner.to_doc().append(" ^ -1"),
            AtomRInner::Sqrt(inner) => RcDoc::text("sqrt(")
                .append(RcDoc::line().append(inner.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
        }
    }
}

impl<S> ToDoc for PatternRInner<S> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.patterns.iter().map(PatTensorR::to_doc),
            RcDoc::line().append(". "),
        )
        .group()
    }
}

impl<S> ToDoc for PatTensorRInner<S> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.patterns.iter().map(PatAtomR::to_doc),
            RcDoc::line().append("x "),
        )
        .group()
    }
}

impl<S> ToDoc for PatAtomRInner<S> {
    fn to_doc(&self) -> RcDoc {
        match self {
            PatAtomRInner::Brackets(pattern) => RcDoc::text("(")
                .append(RcDoc::line().append(pattern.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
            PatAtomRInner::Ket(states) => RcDoc::text(format!(
                "|{}>",
                states.iter().map(KetState::to_char).collect::<String>()
            )),
            PatAtomRInner::Unitary(inner) => inner.to_doc(),
        }
    }
}
