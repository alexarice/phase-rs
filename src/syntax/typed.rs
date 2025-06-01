use std::iter::Sum;

use super::{KetState, Phase};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TermType(pub usize);

impl Sum for TermType {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        TermType(iter.map(|x| x.0).sum())
    }
}

impl TermType {
    pub fn to_pattern_type(self) -> PatternType {
        PatternType(self.0, self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TermT {
    Comp {
        terms: Vec<TermT>,
        ty: TermType,
    },
    Tensor {
        terms: Vec<TermT>,
    },
    Id {
        ty: TermType,
    },
    Phase {
        phase: Phase,
    },
    IfLet {
        pattern: PatternT,
        inner: Box<TermT>,
    },
    Hadamard,
    Gate {
        name: String,
        def: Box<TermT>,
    },
    Inverse {
        inner: Box<TermT>,
    },
}

impl TermT {
    pub fn get_type(&self) -> TermType {
        match self {
            TermT::Comp { ty, .. } => *ty,
            TermT::Tensor { terms } => terms.iter().map(TermT::get_type).sum(),
            TermT::Id { ty } => *ty,
            TermT::Phase { .. } => TermType(0),
            TermT::IfLet { pattern, .. } => TermType(pattern.get_type().0),
            TermT::Hadamard => TermType(1),
            TermT::Gate { def, .. } => def.get_type(),
            TermT::Inverse { inner } => inner.get_type(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatternType(pub usize, pub usize);

impl Sum for PatternType {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(PatternType(0, 0), |PatternType(a, b), PatternType(c, d)| {
            PatternType(a + c, b + d)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternT {
    Comp { patterns: Vec<PatternT> },
    Tensor { patterns: Vec<PatternT> },
    Ket { state: KetState },
    Unitary(Box<TermT>),
}

impl PatternT {
    pub fn get_type(&self) -> PatternType {
        match self {
            PatternT::Comp { patterns } => PatternType(
                patterns.first().unwrap().get_type().0,
                patterns.last().unwrap().get_type().1,
            ),
            PatternT::Tensor { patterns } => patterns.iter().map(PatternT::get_type).sum(),
            PatternT::Ket { .. } => PatternType(1, 0),
            PatternT::Unitary(term) => term.get_type().to_pattern_type(),
        }
    }
}
