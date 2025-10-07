//! Raw syntax patterns.

use std::ops::Range;

use pretty::RcDoc;
use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::multispace0,
    combinator::{alt, delimited, separated},
};

use crate::{
    ket::CompKetState,
    raw_syntax::TermR,
    text::{HasParser, Spanned, ToDoc},
    typecheck::{Env, TypeCheckError},
    typed_syntax::PatternT,
};

/// Raw syntax pattern with text span.
/// Represents a list of tensored patterns composed together.
pub type PatternR<S> = Spanned<S, PatternRInner<S>>;

/// Raw syntax pattern without text span.
/// Represents a list of tensored patterns composed together.
#[derive(Clone, Debug, PartialEq)]
pub struct PatternRInner<S> {
    pub(crate) patterns: Vec<PatTensorR<S>>,
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

/// Raw syntax tensored pattern with text span.
/// Represents a list of pattern atoms tensored together.
pub type PatTensorR<S> = Spanned<S, PatTensorRInner<S>>;

/// Raw syntax tensored pattern without text span.
/// Represents a list of pattern atoms tensored together.
#[derive(Clone, Debug, PartialEq)]
pub struct PatTensorRInner<S> {
    pub(crate) patterns: Vec<PatAtomR<S>>,
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
    Ket(CompKetState),
    /// A unitary pattern
    Unitary(Box<TermR<S>>),
}

impl<S> ToDoc for PatAtomRInner<S> {
    fn to_doc(&self) -> RcDoc {
        match self {
            PatAtomRInner::Brackets(pattern) => RcDoc::text("(")
                .append(RcDoc::line().append(pattern.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
            PatAtomRInner::Ket(states) => states.to_doc(),
            PatAtomRInner::Unitary(inner) => inner.to_doc(),
        }
    }
}

impl<S: Clone> PatternR<S> {
    /// Typecheck a raw pattern in given environment
    pub fn check(&self, env: &Env) -> Result<PatternT, TypeCheckError<S>> {
        let mut pattern_iter = self.inner.patterns.iter();
        let mut raw = pattern_iter.next().unwrap();
        let p = raw.check(env)?;
        let mut ty1 = p.get_type();
        let mut v = vec![p];
        for r in pattern_iter {
            let pattern = r.check(env)?;
            let ty2 = pattern.get_type();
            if ty1.1 != ty2.0 {
                return Err(TypeCheckError::PatternTypeMismatch {
                    p1: raw.clone(),
                    ty1,
                    p2: r.clone(),
                    ty2,
                });
            }
            raw = r;
            ty1 = ty2;
            v.push(pattern);
        }
        Ok(PatternT::Comp(v))
    }
}

impl<S: Clone> PatTensorR<S> {
    fn check(&self, env: &Env) -> Result<PatternT, TypeCheckError<S>> {
        Ok(PatternT::Tensor(
            self.inner
                .patterns
                .iter()
                .map(|p| p.check(env))
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl<S: Clone> PatAtomR<S> {
    fn check(&self, env: &Env) -> Result<PatternT, TypeCheckError<S>> {
        match &self.inner {
            PatAtomRInner::Brackets(pattern) => pattern.check(env),
            PatAtomRInner::Ket(states) => Ok(PatternT::Ket(states.clone())),
            PatAtomRInner::Unitary(inner) => {
                Ok(PatternT::Unitary(Box::new(inner.check(env, None)?)))
            }
        }
    }
}

impl HasParser for PatternRInner<Range<usize>> {
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        separated(1.., PatTensorR::parser, (multispace0, '.', multispace0))
            .map(|patterns| PatternRInner { patterns })
            .parse_next(input)
    }
}

impl HasParser for PatTensorRInner<Range<usize>> {
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        separated(1.., PatAtomR::parser, (multispace0, 'x', multispace0))
            .map(|patterns| PatTensorRInner { patterns })
            .parse_next(input)
    }
}

impl HasParser for PatAtomRInner<Range<usize>> {
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        alt((
            delimited(("(", multispace0), PatternR::parser, (multispace0, ")"))
                .map(PatAtomRInner::Brackets),
            CompKetState::parser.map(PatAtomRInner::Ket),
            TermR::parser.map(|x| PatAtomRInner::Unitary(Box::new(x))),
        ))
        .parse_next(input)
    }
}
