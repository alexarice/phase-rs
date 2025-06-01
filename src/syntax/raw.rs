use super::{KetState, Phase};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeR<S> {
    Unitary(usize, S),
    Pattern(usize, usize, S),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TermR<S> {
    Comp {
        terms: Vec<TermR<S>>,
        span: S,
    },
    Tensor {
        terms: Vec<TermR<S>>,
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
        inner: Box<TermR<S>>,
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
        inner: Box<TermR<S>>,
        span: S,
    },
    Sqrt {
        inner: Box<TermR<S>>,
        span: S,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternR<S> {
    Comp { patterns: Vec<PatternR<S>>, span: S },
    Tensor { patterns: Vec<PatternR<S>>, span: S },
    Ket { states: Vec<KetState>, span: S },
    Unitary(Box<TermR<S>>),
}
