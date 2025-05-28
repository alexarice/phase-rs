use super::KetState;

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
        angle: f64,
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternR<S> {
    Comp { patterns: Vec<PatternR<S>>, span: S },
    Tensor { patterns: Vec<PatternR<S>>, span: S },
    Ket { states: Vec<KetState>, span: S },
    Unitary(Box<TermR<S>>),
}
