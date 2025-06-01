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
    pub fn eval<B: Buildable>(&self) -> B {
        self.eval_with_phase_mul(1.0)
    }

    fn eval_with_phase_mul<B: Buildable>(&self, phase_mul: f64) -> B {
        match self {
            TermT::Comp { terms, ty } => {
                let mapped_terms = terms.iter().map(|t| t.eval_with_phase_mul(phase_mul));
                if phase_mul < 0.0 {
                    B::comp(mapped_terms, ty)
                } else {
                    B::comp(mapped_terms.rev(), ty)
                }
            }
            TermT::Tensor { terms } => {
                B::tensor(terms.iter().map(|t| t.eval_with_phase_mul(phase_mul)))
            }
            TermT::Id { ty } => B::comp(std::iter::empty(), ty),
            TermT::Phase { phase } => B::atom(AtomN::Phase {
                angle: phase_mul * phase.eval(),
            }),
            TermT::IfLet { pattern, inner } => B::atom(AtomN::IfLet {
                pattern: pattern.eval(),
                inner: Box::new(inner.eval_with_phase_mul(phase_mul)),
                ty: pattern.get_type().0,
            }),
            TermT::Hadamard => B::atom(AtomN::Hadamard),
            TermT::Gate { def, .. } => def.eval_with_phase_mul(phase_mul),
            TermT::Inverse { inner } => inner.eval_with_phase_mul(-phase_mul),
            TermT::Sqrt { inner } => inner.eval_with_phase_mul(phase_mul / 2.0),
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
            PatternT::Ket { states } => PatternN::Tensor {
                patterns: states
                    .iter()
                    .map(|&state| PatternN::Ket { state })
                    .collect(),
            },
            PatternT::Unitary(term_t) => term_t.eval(),
        }
    }
}

impl TermN {
    pub fn quote(&self) -> TermT {
        match self {
            TermN::Comp { terms, ty } => TermT::Comp {
                terms: terms.iter().map(TermN::quote).collect(),
                ty: *ty,
            },
            TermN::Tensor { terms } => TermT::Tensor {
                terms: terms.iter().map(TermN::quote).collect(),
            },
            TermN::Atom { atom } => atom.quote(),
        }
    }
}

impl AtomN {
    pub fn quote(&self) -> TermT {
        match self {
            AtomN::Phase { angle } => TermT::Phase {
                phase: Phase::Angle(*angle),
            },
            AtomN::IfLet { pattern, inner, .. } => TermT::IfLet {
                pattern: pattern.quote(),
                inner: Box::new(inner.quote()),
            },
            AtomN::Hadamard => TermT::Hadamard,
        }
    }
}

impl PatternN {
    pub fn quote(&self) -> PatternT {
        match self {
            PatternN::Comp { patterns, .. } => PatternT::Comp {
                patterns: patterns.iter().map(PatternN::quote).collect(),
            },
            PatternN::Tensor { patterns } => PatternT::Tensor {
                patterns: patterns.iter().map(PatternN::quote).collect(),
            },
            PatternN::Ket { state } => PatternT::Ket {
                states: vec![*state],
            },
            PatternN::Unitary(atom_n) => PatternT::Unitary(Box::new(atom_n.quote())),
        }
    }
}
