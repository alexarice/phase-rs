use crate::syntax::{
    normal::{AtomN, PatternN, TermN},
    typed::{PatternT, TermT},
};

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
            TermT::Phase { angle } => TermN::Atom {
                atom: AtomN::Phase { angle: *angle },
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
