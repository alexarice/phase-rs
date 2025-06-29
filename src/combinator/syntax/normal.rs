use super::{
    KetState,
    typed::{PatternType, TermType},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TermN {
    Comp { terms: Vec<TermN>, ty: TermType },
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

impl AtomN {
    pub fn get_type(&self) -> TermType {
        match self {
            AtomN::Phase { .. } => TermType(0),
            AtomN::IfLet { ty, .. } => TermType(*ty),
        }
    }
}
