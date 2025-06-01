use super::{KetState, Phase};

#[derive(Clone, Debug, PartialEq)]
pub struct TermR<S> {
    pub terms: Vec<TensorR<S>>,
    pub span: S,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TensorR<S> {
    pub terms: Vec<AtomR<S>>,
    pub span: S,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AtomR<S> {
    Brackets {
        term: TermR<S>,
        span: S,
    },
    Id {
        qubits: usize,
        span: S,
    },
    Phase {
        phase: Phase,
        span: S,
    },
    IfLet {
        pattern: PatternR<S>,
        inner: Box<AtomR<S>>,
        span: S,
    },
    Hadamard {
        span: S,
    },
    Gate {
        name: String,
        span: S,
    },
    Inverse {
        inner: Box<AtomR<S>>,
        span: S,
    },
    Sqrt {
        inner: TermR<S>,
        span: S,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PatternR<S> {
    pub patterns: Vec<PatTensorR<S>>,
    pub span: S,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PatTensorR<S> {
    pub patterns: Vec<PatAtomR<S>>,
    pub span: S,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatAtomR<S> {
    Brackets { pattern: PatternR<S>, span: S },
    Ket { states: Vec<KetState>, span: S },
    Unitary(Box<TermR<S>>),
}
