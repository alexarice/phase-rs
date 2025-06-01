use crate::syntax::{
    Phase,
    normal::{AtomN, PatternN, TermN},
    typed::{PatternT, PatternType, TermT, TermType},
};

impl Phase {
    pub fn eval(&self) -> f64 {
        match self {
            Phase::Angle(a) => *a,
            Phase::MinusOne => 1.0,
            Phase::Imag => 0.5,
            Phase::NegImag => 1.5,
        }
    }
}

pub trait Buildable {
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self;
    fn tensor(iter: impl Iterator<Item = Self>) -> Self;
    fn atom(atom: AtomN) -> Self;
}

impl Buildable for TermN {
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self {
        TermN::Comp {
            terms: iter.collect(),
            ty: *ty,
        }
    }

    fn tensor(iter: impl Iterator<Item = Self>) -> Self {
        TermN::Tensor {
            terms: iter.collect(),
        }
    }

    fn atom(atom: AtomN) -> Self {
        TermN::Atom { atom }
    }
}

impl Buildable for PatternN {
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self {
        PatternN::Comp {
            patterns: iter.rev().collect(),
            ty: PatternType(ty.0, ty.0),
        }
    }

    fn tensor(iter: impl Iterator<Item = Self>) -> Self {
        PatternN::Tensor {
            patterns: iter.collect(),
        }
    }

    fn atom(atom: AtomN) -> Self {
        PatternN::Unitary(Box::new(atom))
    }
}

impl TermT {
    pub fn eval<B: Buildable>(&self, inv: bool) -> B {
        match self {
            TermT::Comp { terms, ty } => {
                let mapped_terms = terms.iter().map(|t| t.eval(inv));
                if inv {
                    B::comp(mapped_terms, ty)
                } else {
                    B::comp(mapped_terms.rev(), ty)
                }
            }
            TermT::Tensor { terms } => B::tensor(terms.iter().map(|t| t.eval(inv))),
            TermT::Id { ty } => B::comp(std::iter::empty(), ty),
            TermT::Phase { phase } => B::atom(AtomN::Phase {
                angle: if inv { -phase.eval() } else { phase.eval() },
            }),
            TermT::IfLet { pattern, inner } => B::atom(AtomN::IfLet {
                pattern: pattern.eval(),
                inner: Box::new(inner.eval(inv)),
                ty: pattern.get_type().0,
            }),
            TermT::Hadamard => B::atom(AtomN::Hadamard),
            TermT::Gate { def, .. } => def.eval(inv),
            TermT::Inverse { inner } => inner.eval(!inv),
        }
    }
}

impl PatternT {
    pub fn eval(&self) -> PatternN {
        match self {
            PatternT::Comp { patterns } => PatternN::Comp {
                patterns: patterns.iter().map(PatternT::eval).collect(),
                ty: self.get_type(),
            },
            PatternT::Tensor { patterns } => PatternN::Tensor {
                patterns: patterns.iter().map(PatternT::eval).collect(),
            },
            PatternT::Ket { state } => PatternN::Ket { state: *state },
            PatternT::Unitary(term_t) => term_t.eval(false),
        }
    }
}
