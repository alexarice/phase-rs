use super::KetState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeR<S> {
    Unitary(usize, S),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TermR<S> {
    Comp {
        first: Box<TermR<S>>,
        second: Box<TermR<S>>,
        span: S,
    },
    Tensor {
        first: Box<TermR<S>>,
        second: Box<TermR<S>>,
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
    Comp {
        first: Box<PatternR<S>>,
        second: Box<PatternR<S>>,
        span: S,
    },
    Tensor {
        first: Box<PatternR<S>>,
        second: Box<PatternR<S>>,
        span: S,
    },
    Ket {
        states: Vec<KetState>,
        span: S,
    },
    Unitary(Box<TermR<S>>),
}
