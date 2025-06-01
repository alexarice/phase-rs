use std::collections::HashMap;

use crate::syntax::{
    raw::{PatternR, TermR},
    typed::{PatternT, PatternType, TermT, TermType},
};

#[derive(Debug, Clone)]
pub enum TypeCheckError<S> {
    TypeMismatch {
        t1: TermR<S>,
        ty1: TermType,
        t2: TermR<S>,
        ty2: TermType,
    },
    IfTypeMismatch {
        p: PatternR<S>,
        pty: PatternType,
        t: TermR<S>,
        tty: TermType,
    },
    PatternTypeMismatch {
        p1: PatternR<S>,
        ty1: PatternType,
        p2: PatternR<S>,
        ty2: PatternType,
    },
    UnknownSymbol {
        name: String,
        span: S,
    },
    TermNotRootable {
        tm: TermR<S>,
        span_of_root: S,
    },
}

#[derive(Default)]
pub struct Env(pub(crate) HashMap<String, TermT>);

impl<S: Clone> TermR<S> {
    pub fn check(&self, env: &Env, check_sqrt: Option<&S>) -> Result<TermT, TypeCheckError<S>> {
        match self {
            TermR::Comp { terms, .. } => {
                if let Some(span) = check_sqrt {
                    return Err(TypeCheckError::TermNotRootable {
                        tm: self.clone(),
                        span_of_root: span.clone(),
                    });
                }
                let mut term_iter = terms.iter();
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
                Ok(TermT::Comp { terms: v, ty: ty1 })
            }
            TermR::Tensor { terms, .. } => Ok(TermT::Tensor {
                terms: terms
                    .iter()
                    .map(|t| t.check(env, check_sqrt))
                    .collect::<Result<_, _>>()?,
            }),
            TermR::Id { qubits, .. } => Ok(TermT::Comp {
                terms: vec![],
                ty: TermType(*qubits),
            }),
            TermR::Phase { phase, .. } => Ok(TermT::Phase { phase: *phase }),
            TermR::IfLet { pattern, inner, .. } => {
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
            TermR::Hadamard { .. } => Ok(TermT::Hadamard),
            TermR::Gate { name, span } => {
                if let Some(def) = env.0.get(name) {
                    Ok(TermT::Gate {
                        name: name.clone(),
                        def: Box::new(def.clone()),
                    })
                } else {
                    Err(TypeCheckError::UnknownSymbol {
                        name: name.to_owned(),
                        span: span.clone(),
                    })
                }
            }
            TermR::Inverse { inner, .. } => {
                let inner_t = inner.check(env, check_sqrt)?;
                Ok(TermT::Inverse {
                    inner: Box::new(inner_t),
                })
            }
            TermR::Sqrt { inner, span } => {
                let inner_t = if check_sqrt.is_some() {
                    inner.check(env, None)?
                } else {
                    inner.check(env, Some(span))?
                };

                Ok(TermT::Sqrt {
                    inner: Box::new(inner_t),
                })
            }
        }
    }
}

impl<S: Clone> PatternR<S> {
    pub fn check(&self, env: &Env) -> Result<PatternT, TypeCheckError<S>> {
        match self {
            PatternR::Comp { patterns, .. } => {
                let mut pattern_iter = patterns.iter();
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
                Ok(PatternT::Comp { patterns: v })
            }
            PatternR::Tensor { patterns, .. } => Ok(PatternT::Tensor {
                patterns: patterns
                    .iter()
                    .map(|p| p.check(env))
                    .collect::<Result<_, _>>()?,
            }),
            PatternR::Ket { states, .. } => Ok(PatternT::Tensor {
                patterns: states
                    .iter()
                    .map(|&state| PatternT::Ket { state })
                    .collect(),
            }),
            PatternR::Unitary(term_r) => Ok(PatternT::Unitary(Box::new(term_r.check(env, None)?))),
        }
    }
}
