//! Helpers for parsing and pretty printing.

use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use miette::SourceSpan;
use pretty::RcDoc;
use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::{alphanumeric1, multispace0},
    combinator::repeat,
    error::{StrContext, StrContextValue},
    token::take_until,
};

/// Trait for types which can be pretty-printed
pub trait ToDoc {
    /// Produce an `RcDoc` for pretty-printing.
    fn to_doc(&self) -> RcDoc<'_>;
}

/// Trait for types which can be parsed
pub trait HasParser: Sized {
    /// Parse an element of this type.
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self>;
}

pub trait Span: Clone + Debug + Into<SourceSpan> {}

impl Span for Range<usize> {}

#[derive(Clone, Debug)]
pub struct NoSpan;

impl From<NoSpan> for SourceSpan {
    fn from(_: NoSpan) -> Self {
        0.into()
    }
}

impl Span for NoSpan {}

/// Wraps data of type `T` in a span of type `S`, locating it in the source text.
/// The span is ignored when printing.
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<S, T> {
    /// Wrapped data
    pub inner: T,
    /// Text span
    pub span: S,
}

impl<S, T: ToDoc> ToDoc for Spanned<S, T> {
    fn to_doc(&self) -> RcDoc<'_> {
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

impl<S: Span, T> From<Spanned<S, T>> for SourceSpan {
    fn from(value: Spanned<S, T>) -> Self {
        value.span.into()
    }
}
impl<S: Span, T: Clone + Debug> Span for Spanned<S, T> {}

impl<T: HasParser> HasParser for Spanned<Range<usize>, T> {
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        T::parser
            .with_span()
            .map(|(inner, span)| Spanned { inner, span })
            .parse_next(input)
    }
}

/// Parse a comment
pub fn comment_parser(input: &mut LocatingSlice<&str>) -> ModalResult<()> {
    (
        multispace0,
        repeat::<_, _, (), _, _>(0.., ("//", take_until(0.., "\n"), multispace0).value(())),
    )
        .parse_next(input)?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// An identifier
pub struct Name(String);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl ToDoc for Name {
    fn to_doc(&self) -> RcDoc<'_> {
        RcDoc::text(&self.0)
    }
}

impl HasParser for Name {
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        alphanumeric1
            .map(|s: &str| Name(s.to_owned()))
            .context(StrContext::Label("identifier"))
            .context(StrContext::Expected(StrContextValue::Description(
                "alphanumeric string",
            )))
            .parse_next(input)
    }
}
