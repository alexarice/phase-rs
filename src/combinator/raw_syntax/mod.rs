//! Raw syntax definitions
//!
//! Raw syntax is used primarily for parsing and printed.
//! It is not assumed to be typechecked/well-formed.

use std::borrow::Cow;

use pretty::RcDoc;

use crate::{
    combinator::{typecheck::{Env, TypeCheckError}, typed_syntax::{PatternT, TermT, TermType}}, ket::KetState, phase::Phase, text::{Spanned, ToDoc}
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
            PatAtomRInner::Ket(states) => states.to_doc(),
            PatAtomRInner::Unitary(inner) => inner.to_doc(),
        }
    }
}

impl<S: Clone> TermR<S> {
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

impl<S: Clone> TensorR<S> {
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

impl<S: Clone> AtomR<S> {
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
                        name: name.to_owned(),
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
