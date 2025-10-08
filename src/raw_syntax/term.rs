//! Raw syntax terms.

use std::ops::Range;

use pretty::RcDoc;
use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::{dec_uint, multispace0, multispace1},
    combinator::{alt, cut_err, delimited, opt, preceded, separated, seq},
    error::{StrContext, StrContextValue},
};

use crate::{
    phase::Phase,
    raw_syntax::PatternR,
    text::{HasParser, Name, Span, Spanned, ToDoc},
    typecheck::{Env, TypeCheckError},
    typed_syntax::{TermT, TermType},
};

/// Raw syntax term with text span.
/// Represents a list of tensored terms composed together.
pub type TermR<S> = Spanned<S, TermRInner<S>>;

/// Raw syntax term without text span.
/// Represents a list of tensored terms composed together.
#[derive(Clone, Debug, PartialEq)]
pub struct TermRInner<S> {
    pub(crate) terms: Vec<TensorR<S>>,
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

/// Raw syntax tensored term with text span.
/// Represents a list of atoms tensored together.
pub type TensorR<S> = Spanned<S, TensorRInner<S>>;

/// Raw syntax tensored term without text span.
/// Represents a list of atoms tensored together.
#[derive(Clone, Debug, PartialEq)]
pub struct TensorRInner<S> {
    pub(crate) terms: Vec<AtomR<S>>,
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
        inner: Box<TensorR<S>>,
    },
    /// Top level symbol, a named gate
    Gate(Name),
    /// Inverse of a term "t ^ -1"
    Inverse(Box<AtomR<S>>),
    /// Square root of a term "sqrt(t)"
    Sqrt(Box<AtomR<S>>),
}

impl<S> ToDoc for AtomRInner<S> {
    fn to_doc(&self) -> RcDoc {
        match self {
            AtomRInner::Brackets(term) => RcDoc::text("(")
                .append(RcDoc::line().append(term.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
            AtomRInner::Id(qubits) => RcDoc::text("id").append(if *qubits == 1 {
                RcDoc::nil()
            } else {
                RcDoc::as_string(qubits)
            }),
            AtomRInner::Phase(phase) => phase.to_doc(),
            AtomRInner::IfLet { pattern, inner, .. } => RcDoc::text("if let")
                .append(RcDoc::line().append(pattern.to_doc()).nest(2))
                .append(RcDoc::line())
                .append("then")
                .group()
                .append(RcDoc::line().append(inner.to_doc()).nest(2))
                .group(),
            AtomRInner::Gate(name) => name.to_doc(),
            AtomRInner::Inverse(inner) => inner.to_doc().append(" ^ -1"),
            AtomRInner::Sqrt(inner) => RcDoc::text("sqrt(")
                .append(RcDoc::line().append(inner.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
        }
    }
}

impl<S: Span> TermR<S> {
    /// Typecheck a raw term in given environment
    /// If `check_sqrt` is not `None`, then checks that the term is "composition free"
    pub fn check(&self, env: &Env, check_sqrt: Option<&S>) -> Result<TermT, TypeCheckError<S>> {
        if let Some(span) = check_sqrt {
            if self.inner.terms.len() != 1 {
                return Err(TypeCheckError::TermNotRootable {
                    tm: self.clone(),
                    span_of_root: span.clone(),
                });
            }
        }
        let mut term_iter = self.inner.terms.iter();
        let mut raw = term_iter.next().unwrap();
        let t = raw.check(env, check_sqrt)?;
        let ty1 = t.get_type();
        let mut v = vec![t];
        for r in term_iter {
            let term = r.check(env, check_sqrt)?;
            let ty2 = term.get_type();
            if ty1 != ty2 {
                return Err(TypeCheckError::TypeMismatch {
                    t1: raw.clone(),
                    ty1,
                    t2: r.clone(),
                    ty2,
                });
            }
            raw = r;
            v.push(term);
        }
        Ok(TermT::Comp(v))
    }
}

impl<S: Span> TensorR<S> {
    fn check(&self, env: &Env, check_sqrt: Option<&S>) -> Result<TermT, TypeCheckError<S>> {
        Ok(TermT::Tensor(
            self.inner
                .terms
                .iter()
                .map(|t| t.check(env, check_sqrt))
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl<S: Span> AtomR<S> {
    fn check(&self, env: &Env, check_sqrt: Option<&S>) -> Result<TermT, TypeCheckError<S>> {
        match &self.inner {
            AtomRInner::Brackets(term) => term.check(env, check_sqrt),
            AtomRInner::Id(qubits) => Ok(TermT::Id(TermType(*qubits))),
            AtomRInner::Phase(phase) => Ok(TermT::Phase(*phase)),
            AtomRInner::IfLet { pattern, inner, .. } => {
                let p = pattern.check(env)?;
                let t = inner.check(env, check_sqrt)?;
                let pty = p.get_type();
                let tty = t.get_type();
                if pty.1 != tty.0 {
                    Err(TypeCheckError::IfTypeMismatch {
                        p: pattern.clone(),
                        pty,
                        t: inner.as_ref().clone(),
                        tty,
                    })
                } else {
                    Ok(TermT::IfLet {
                        pattern: p,
                        inner: Box::new(t),
                    })
                }
            }
            AtomRInner::Gate(name) => {
                if let Some(def) = env.0.get(name) {
                    Ok(TermT::Gate {
                        name: name.clone(),
                        def: Box::new(def.clone()),
                    })
                } else {
                    Err(TypeCheckError::UnknownSymbol {
                        name: name.clone(),
                        span: self.span.clone(),
                    })
                }
            }
            AtomRInner::Inverse(inner) => {
                let inner_t = inner.check(env, check_sqrt)?;
                Ok(TermT::Inverse(Box::new(inner_t)))
            }
            AtomRInner::Sqrt(inner) => {
                let inner_t = if check_sqrt.is_some() {
                    inner.check(env, None)?
                } else {
                    inner.check(env, Some(&self.span))?
                };

                Ok(TermT::Sqrt(Box::new(inner_t)))
            }
        }
    }
}

impl HasParser for TermRInner<Range<usize>> {
    /// Parser for terms.
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        separated(1.., TensorR::parser, (multispace0, ';', multispace0))
            .context(StrContext::Label("term"))
            .map(|terms| TermRInner { terms })
            .parse_next(input)
    }
}

impl HasParser for TensorRInner<Range<usize>> {
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        separated(1.., AtomR::parser, (multispace0, 'x', multispace0))
            .context(StrContext::Label("term"))
            .map(|terms| TensorRInner { terms })
            .parse_next(input)
    }
}

impl HasParser for AtomRInner<Range<usize>> {
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        let without_inverse = alt((
            delimited(
                ("(", multispace0),
                cut_err(TermR::parser),
                cut_err(
                    (multispace0, ")")
                        .context(StrContext::Expected(StrContextValue::CharLiteral(')'))),
                ),
            )
            .map(AtomRInner::Brackets),
            preceded(("sqrt", multispace0), cut_err(AtomR::parser))
                .map(|inner| AtomRInner::Sqrt(Box::new(inner))),
            preceded("id", opt(dec_uint)).map(|qubits| AtomRInner::Id(qubits.unwrap_or(1))),
            preceded(
                "if",
                cut_err(seq!(
		_: multispace1,
		_: "let".context(StrContext::Expected(StrContextValue::StringLiteral("let"))),
		_: multispace1,
		PatternR::parser,
		_: multispace1,
		_: "then".context(StrContext::Expected(StrContextValue::StringLiteral("then"))),
		_: multispace1,
		TensorR::parser)),
            )
            .map(|(pattern, inner)| AtomRInner::IfLet {
                pattern,
                inner: Box::new(inner),
            }),
            Phase::parser.map(AtomRInner::Phase),
            Name::parser.map(AtomRInner::Gate),
        ))
        .context(StrContext::Expected(StrContextValue::CharLiteral('(')))
        .context(StrContext::Expected(StrContextValue::StringLiteral("sqrt")))
        .context(StrContext::Expected(StrContextValue::StringLiteral("id")))
        .context(StrContext::Expected(StrContextValue::StringLiteral("if")))
        .context(StrContext::Expected(StrContextValue::CharLiteral('H')))
        .context(StrContext::Expected(StrContextValue::Description(
            "identifier",
        )));

        (
            without_inverse,
            opt((
                multispace0,
                "^",
                multispace0,
                cut_err("-1").context(StrContext::Expected(StrContextValue::StringLiteral("-1"))),
            ))
            .context(StrContext::Label("term")),
        )
            .with_span()
            .map(|((inner, invert), span)| {
                if invert.is_some() {
                    AtomRInner::Inverse(Box::new(Spanned { inner, span }))
                } else {
                    inner
                }
            })
            .parse_next(input)
    }
}
