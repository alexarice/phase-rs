use std::ops::Range;

use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::{dec_uint, multispace0},
    combinator::{alt, cut_err, delimited, opt, preceded, separated},
    error::{StrContext, StrContextValue},
};

use super::syntax::raw::{Copattern, CopatternInner, TypeR, TypeRInner};
use crate::common::{Spanned, identifier, parse_spanned};

pub fn ty(input: &mut LocatingSlice<&str>) -> ModalResult<TypeR<Range<usize>>> {
    parse_spanned(preceded('q', dec_uint).map(|i| TypeRInner(i))).parse_next(input)
}

pub fn copattern(input: &mut LocatingSlice<&str>) -> ModalResult<Copattern<Range<usize>>> {
    separated(1.., copattern_atom, (multispace0, 'x', multispace0))
        .with_span()
        .map(|(copatterns, span): (Vec<_>, _)| {
            if copatterns.len() == 1 {
                copatterns.into_iter().next().unwrap()
            } else {
                Spanned {
                    inner: CopatternInner::Tensor { copatterns },
                    span,
                }
            }
        })
        .parse_next(input)
}

pub fn copattern_atom(input: &mut LocatingSlice<&str>) -> ModalResult<Copattern<Range<usize>>> {
    (
        alt((
            delimited(
                ('(', multispace0),
                cut_err(copattern),
                cut_err(
                    (multispace0, ')')
                        .context(StrContext::Expected(StrContextValue::CharLiteral(')'))),
                ),
            ),
            parse_spanned(identifier.map(|name| CopatternInner::Var { name })),
        )),
        opt(preceded((multispace0, ';', multispace0), cut_err(ty))),
    )
        .with_span()
        .map(|((inner, t), span)| {
            if let Some(ty) = t {
                Spanned {
                    inner: CopatternInner::Annotated {
                        inner: Box::new(inner),
                        ty,
                    },
                    span,
                }
            } else {
                inner
            }
        })
        .parse_next(input)
}
