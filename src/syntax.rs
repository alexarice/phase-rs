#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeS<S> {
    Unitary(usize, S),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TermS<S> {
    Comp {
        first: Box<TermS<S>>,
        second: Box<TermS<S>>,
        span: S,
    },
    Tensor {
        first: Box<TermS<S>>,
        second: Box<TermS<S>>,
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
        pattern: PatternS<S>,
        inner: Box<TermS<S>>,
        span: S,
    },
    Hadamard {
        span: S,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KetState {
    Zero,
    One,
    Plus,
    Minus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternS<S> {
    Comp {
        first: Box<PatternS<S>>,
        second: Box<PatternS<S>>,
        span: S,
    },
    Tensor {
        first: Box<PatternS<S>>,
        second: Box<PatternS<S>>,
        span: S,
    },
    Ket {
        states: Vec<KetState>,
        span: S,
    },
    Unitary(Box<TermS<S>>),
}
