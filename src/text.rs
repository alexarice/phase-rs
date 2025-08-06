//! Helpers for parsing and pretty printing.

use std::ops::Range;

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
    fn to_doc(&self) -> RcDoc;
}

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

/// Parse a comment
pub fn comment(input: &mut LocatingSlice<&str>) -> ModalResult<()> {
    (
        multispace0,
        repeat::<_, _, (), _, _>(0.., ("//", take_until(0.., "\n"), multispace0).value(())),
    )
        .parse_next(input)?;
    Ok(())
}

/// Parse an identifier
pub fn identifier(input: &mut LocatingSlice<&str>) -> ModalResult<String> {
    alphanumeric1
        .map(|s: &str| s.to_owned())
        .context(StrContext::Label("identifier"))
        .context(StrContext::Expected(StrContextValue::Description(
            "alphanumeric string",
        )))
        .parse_next(input)
}
