//! Functions and datastructures for type checking

use std::collections::HashMap;

use super::{
    raw_syntax::{
        AtomR, AtomRInner, PatAtomR, PatAtomRInner, PatTensorR, PatTensorRInner, PatternR,
        PatternRInner, TensorR, TensorRInner, TermR, TermRInner,
    },
    typed_syntax::{PatternT, PatternType, TermT, TermType},
};

/// Errors that can occur during typechecking.
#[derive(Debug, Clone)]
pub enum TypeCheckError<S> {
    /// Error for mismatching type between terms in a composition.
    TypeMismatch {
        /// Term 1
        t1: TensorR<S>,
        /// Type of term 1
        ty1: TermType,
        /// Term 2
        t2: TensorR<S>,
        /// Type of term 2
        ty2: TermType,
    },
    /// Error for mismatching type between a term and pattern in an "if let" statement.
    IfTypeMismatch {
        /// Pattern
        p: PatternR<S>,
        /// Type of pattern
        pty: PatternType,
        /// Body term
        t: AtomR<S>,
        /// Type of body term
        tty: TermType,
    },
    /// Error for mismatching type between composed patterns.
    PatternTypeMismatch {
        /// Pattern 1
        p1: PatTensorR<S>,
        /// Type of pattern 1
        ty1: PatternType,
        /// Pattern 2
        p2: PatTensorR<S>,
        /// Type of pattern 2
        ty2: PatternType,
    },
    /// Error for an unknown top-level symbol.
    UnknownSymbol {
        /// The unknown symbol encountered
        name: String,
        /// Span of symbol
        span: S,
    },
    /// Error for when a sqrt operation is applied to a term with compositions.
    TermNotRootable {
        /// Term which contains compositions
        tm: TermR<S>,
        /// Span of sqrt term causing error
        span_of_root: S,
    },
}

/// Typing enviroment, holding definitions of top level symbols.
#[derive(Default)]
pub struct Env(pub(crate) HashMap<String, TermT>);

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

impl TermT {
    /// Convert to a raw term.
    pub fn to_raw(&self) -> TermR<()> {
        let terms = if let TermT::Comp(terms) = self {
            terms.iter().map(|t| t.to_raw_tensor()).collect()
        } else {
            vec![self.to_raw_tensor()]
        };
        TermRInner { terms }.into()
    }

    fn to_raw_tensor(&self) -> TensorR<()> {
        let terms = if let TermT::Tensor(terms) = self {
            terms.iter().map(|t| t.to_raw_atom()).collect()
        } else {
            vec![self.to_raw_atom()]
        };
        TensorRInner { terms }.into()
    }

    fn to_raw_atom(&self) -> AtomR<()> {
        match self {
            TermT::Id(ty) => AtomRInner::Id(ty.0),
            TermT::Phase(phase) => AtomRInner::Phase(*phase),
            TermT::IfLet { pattern, inner } => AtomRInner::IfLet {
                pattern: pattern.to_raw(),
                inner: Box::new(inner.to_raw_atom()),
            },
            TermT::Gate { name, .. } => AtomRInner::Gate(name.to_owned()),
            TermT::Inverse(inner) => AtomRInner::Inverse(Box::new(inner.to_raw_atom())),
            TermT::Sqrt(inner) => AtomRInner::Sqrt(Box::new(inner.to_raw_atom())),
            t => AtomRInner::Brackets(t.to_raw()),
        }
        .into()
    }
}

impl PatternT {
    /// Convert to a raw pattern.
    pub fn to_raw(&self) -> PatternR<()> {
        let patterns = if let PatternT::Comp(patterns) = self {
            patterns.iter().map(|t| t.to_raw_tensor()).collect()
        } else {
            vec![self.to_raw_tensor()]
        };
        PatternRInner { patterns }.into()
    }

    fn to_raw_tensor(&self) -> PatTensorR<()> {
        let patterns = if let PatternT::Tensor(patterns) = self {
            patterns.iter().map(|t| t.to_raw_atom()).collect()
        } else {
            vec![self.to_raw_atom()]
        };
        PatTensorRInner { patterns }.into()
    }

    fn to_raw_atom(&self) -> PatAtomR<()> {
        match self {
            PatternT::Ket(states) => PatAtomRInner::Ket(states.clone()),
            PatternT::Unitary(inner) => PatAtomRInner::Unitary(Box::new(inner.to_raw())),
            p => PatAtomRInner::Brackets(p.to_raw()),
        }
        .into()
    }
}
