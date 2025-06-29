use super::syntax::{
    normal::{AtomN, PatternN, TermN},
    typed::{PatternT, PatternType, TermT, TermType},
};
use crate::common::Phase;

impl Phase {
    pub fn eval(&self) -> f64 {
        match self {
            Phase::Angle(a) => *a,
            Phase::MinusOne => 1.0,
            Phase::Imag => 0.5,
            Phase::MinusImag => 1.5,
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
                let mut mapped_terms = terms.iter().map(|t| t.eval_with_phase_mul(phase_mul));
                if terms.len() == 1 {
                    mapped_terms.next().unwrap()
                } else if phase_mul < 0.0 {
                    B::comp(mapped_terms, ty)
                } else {
                    B::comp(mapped_terms.rev(), ty)
                }
            }
            TermT::Tensor { terms } => {
                if terms.len() == 1 {
                    terms[0].eval_with_phase_mul(phase_mul)
                } else {
                    B::tensor(terms.iter().map(|t| t.eval_with_phase_mul(phase_mul)))
                }
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
            TermT::Gate { def, .. } => def.eval_with_phase_mul(phase_mul),
            TermT::Inverse { inner } => inner.eval_with_phase_mul(-phase_mul),
            TermT::Sqrt { inner } => inner.eval_with_phase_mul(phase_mul / 2.0),
        }
    }
}

impl PatternT {
    pub fn eval(&self) -> PatternN {
        match self {
            PatternT::Comp { patterns } => {
                if patterns.len() == 1 {
                    patterns[0].eval()
                } else {
                    PatternN::Comp {
                        patterns: patterns.iter().map(PatternT::eval).collect(),
                        ty: self.get_type(),
                    }
                }
            }
            PatternT::Tensor { patterns } => {
                if patterns.len() == 1 {
                    patterns[0].eval()
                } else {
                    PatternN::Tensor {
                        patterns: patterns.iter().map(PatternT::eval).collect(),
                    }
                }
            }
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
            TermN::Comp { terms, ty } => {
                if terms.is_empty() {
                    TermT::Id { ty: *ty }
                } else {
                    TermT::Comp {
                        terms: terms.iter().map(TermN::quote).collect(),
                        ty: *ty,
                    }
                }
            }
            TermN::Tensor { terms } => TermT::Tensor {
                terms: terms.iter().map(TermN::quote).collect(),
            },
            TermN::Atom { atom } => atom.quote(),
        }
    }

    pub fn squash_comp(mut self, acc: &mut Vec<TermN>) {
        if let TermN::Comp { terms, .. } = self {
            for t in terms {
                t.squash_comp(acc);
            }
        } else {
            self.squash();
            acc.push(self);
        }
    }

    pub fn squash_tensor(mut self, acc: &mut Vec<TermN>) {
        if let TermN::Tensor { terms, .. } = self {
            for t in terms {
                t.squash_tensor(acc);
            }
        } else {
            self.squash();
            acc.push(self);
        }
    }

    pub fn squash(&mut self) {
        match self {
            TermN::Comp { terms, .. } => {
                let old_terms = std::mem::take(terms);
                for t in old_terms {
                    t.squash_comp(terms);
                }
                if terms.len() == 1 {
                    *self = terms.pop().unwrap();
                }
            }
            TermN::Tensor { terms } => {
                let old_terms = std::mem::take(terms);
                for t in old_terms {
                    t.squash_tensor(terms);
                }
                if terms.len() == 1 {
                    *self = terms.pop().unwrap();
                }
            }
            TermN::Atom { atom } => atom.squash(),
        }
    }
}

impl AtomN {
    pub fn quote(&self) -> TermT {
        match self {
            AtomN::Phase { angle } => TermT::Phase {
                phase: Phase::from_angle(*angle),
            },
            AtomN::IfLet { pattern, inner, .. } => TermT::IfLet {
                pattern: pattern.quote(),
                inner: Box::new(inner.quote()),
            },
        }
    }

    pub fn squash(&mut self) {
        if let AtomN::IfLet { pattern, inner, .. } = self {
            pattern.squash();
            inner.squash();
        }
    }
}

impl PatternN {
    pub fn quote(&self) -> PatternT {
        match self {
            PatternN::Comp { patterns, ty } => {
                if patterns.is_empty() {
                    PatternT::Unitary(Box::new(TermT::Id { ty: TermType(ty.0) }))
                } else {
                    PatternT::Comp {
                        patterns: patterns.iter().map(PatternN::quote).collect(),
                    }
                }
            }
            PatternN::Tensor { patterns } => PatternT::Tensor {
                patterns: patterns.iter().map(PatternN::quote).collect(),
            },
            PatternN::Ket { state } => PatternT::Ket {
                states: vec![*state],
            },
            PatternN::Unitary(atom_n) => PatternT::Unitary(Box::new(atom_n.quote())),
        }
    }

    pub fn squash_comp(mut self, acc: &mut Vec<PatternN>) {
        if let PatternN::Comp { patterns, .. } = self {
            for p in patterns {
                p.squash_comp(acc);
            }
        } else {
            self.squash();
            acc.push(self);
        }
    }

    pub fn squash_tensor(mut self, acc: &mut Vec<PatternN>) {
        if let PatternN::Tensor { patterns, .. } = self {
            for p in patterns {
                p.squash_tensor(acc);
            }
        } else {
            self.squash();
            acc.push(self);
        }
    }

    pub fn squash(&mut self) {
        match self {
            PatternN::Comp { patterns, .. } => {
                let old_patterns = std::mem::take(patterns);
                for p in old_patterns {
                    p.squash_comp(patterns);
                }
                if patterns.len() == 1 {
                    *self = patterns.pop().unwrap();
                }
            }
            PatternN::Tensor { patterns } => {
                let old_patterns = std::mem::take(patterns);
                for p in old_patterns {
                    p.squash_tensor(patterns);
                }
                if patterns.len() == 1 {
                    *self = patterns.pop().unwrap();
                }
            }
            PatternN::Ket { .. } => {}
            PatternN::Unitary(atom_n) => atom_n.squash(),
        }
    }
}
