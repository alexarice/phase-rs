

/// Wraps data of type `T` in a span of type `S`, locating it in the source text.
/// The span is ignored when printing.
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<S, T> {
    pub inner: T,
    pub span: S,
}

pub fn parse_spanned<'a, T, E>(
    inner: impl Parse<LocatingSlice<&'a str>, T, E>,
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
