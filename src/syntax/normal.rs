use super::{
    KetState,
    typed::{PatternType, TermType},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TermN {
    Comp { terms: Vec<TermN>, ty: usize },
    Tensor { terms: Vec<TermN> },
    Atom { atom: AtomN },
}

#[derive(Clone, Debug, PartialEq)]
pub enum AtomN {
    Phase {
        angle: f64,
    },
    IfLet {
        pattern: PatternN,
        inner: Box<TermN>,
        ty: usize,
    },
    Hadamard,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternN {
    Comp {
        patterns: Vec<PatternN>,
        ty: PatternType,
    },
    Tensor {
        patterns: Vec<PatternN>,
    },
    Ket {
        state: KetState,
    },
    Unitary(Box<AtomN>),
}

impl TermN {
    pub fn to_pattern(self) -> PatternN {
        match self {
            TermN::Comp { terms, ty } => PatternN::Comp {
                patterns: terms.into_iter().map(TermN::to_pattern).collect(),
                ty: PatternType(ty, ty),
            },
            TermN::Tensor { terms } => PatternN::Tensor {
                patterns: terms.into_iter().map(TermN::to_pattern).collect(),
            },
            TermN::Atom { atom } => PatternN::Unitary(Box::new(atom)),
        }
    }
}

impl AtomN {
    pub fn get_type(&self) -> TermType {
        match self {
            AtomN::Phase { .. } => TermType(0),
            AtomN::IfLet { ty, .. } => TermType(*ty),
            AtomN::Hadamard => TermType(1),
        }
    }
}
