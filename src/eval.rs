use crate::syntax::{
    Phase,
    normal::{AtomN, PatternN, TermN},
    typed::{PatternT, TermT},
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

impl TermT {
    pub fn eval(&self) -> TermN {
        match self {
            TermT::Comp { terms, ty } => TermN::Comp {
                terms: terms.iter().map(TermT::eval).collect(),
                ty: *ty,
            },
            TermT::Tensor { terms } => TermN::Tensor {
                terms: terms.iter().map(TermT::eval).collect(),
            },
            TermT::Id { ty } => TermN::Comp {
                terms: vec![],
                ty: *ty,
            },
            TermT::Phase { phase } => TermN::Atom {
                atom: AtomN::Phase {
                    angle: phase.eval(),
                },
            },
            TermT::IfLet { pattern, inner } => TermN::Atom {
                atom: AtomN::IfLet {
                    pattern: pattern.eval(),
                    inner: Box::new(inner.eval()),
                    ty: pattern.get_type().0,
                },
            },
            TermT::Hadamard => TermN::Atom {
                atom: AtomN::Hadamard,
            },
            TermT::Gate { def, .. } => def.eval(),
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
            PatternT::Ket { state } => PatternN::Ket { state: *state },
            PatternT::Unitary(term_t) => term_t.eval().to_pattern(),
        }
    }
}
