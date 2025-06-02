use pretty::RcDoc;

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

impl<S> TermR<S> {
    pub fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.terms.iter().map(TensorR::to_doc),
            RcDoc::text(";").append(RcDoc::line()),
        )
        .group()
    }
}

impl<S> TensorR<S> {
    pub fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.terms.iter().map(AtomR::to_doc),
            RcDoc::line().append("x "),
        )
        .group()
    }
}

impl<S> AtomR<S> {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            AtomR::Brackets { term, .. } => RcDoc::text("(")
                .append(RcDoc::line().append(term.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
            AtomR::Id { qubits, .. } => RcDoc::text(format!("id{qubits}")),
            AtomR::Phase { phase, .. } => phase.to_doc(),
            AtomR::IfLet { pattern, inner, .. } => RcDoc::text("if let")
                .append(RcDoc::line().append(pattern.to_doc()).nest(2))
                .append(RcDoc::line())
                .append("then")
                .group()
                .append(RcDoc::line().append(inner.to_doc()).nest(2))
                .group(),
            AtomR::Hadamard { .. } => RcDoc::text("H"),
            AtomR::Gate { name, .. } => RcDoc::text(name),
            AtomR::Inverse { inner, .. } => inner.to_doc().append(" ^ -1"),
            AtomR::Sqrt { inner, .. } => RcDoc::text("sqrt(")
                .append(RcDoc::line().append(inner.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
        }
    }
}

impl Phase {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            Phase::Angle(a) => RcDoc::text(format!("ph({a}pi)")),
            Phase::MinusOne => RcDoc::text("-1"),
            Phase::Imag => RcDoc::text("i"),
            Phase::NegImag => RcDoc::text("-i"),
        }
    }
}

impl<S> PatternR<S> {
    pub fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.patterns.iter().map(PatTensorR::to_doc),
            RcDoc::line().append(". "),
        )
        .group()
    }
}

impl<S> PatTensorR<S> {
    pub fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.patterns.iter().map(PatAtomR::to_doc),
            RcDoc::line().append("x "),
        )
        .group()
    }
}

impl<S> PatAtomR<S> {
    pub fn to_doc(&self) -> RcDoc {
        match self {
            PatAtomR::Brackets { pattern, .. } => RcDoc::text("(")
                .append(RcDoc::line().append(pattern.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
            PatAtomR::Ket { states, .. } => RcDoc::text(format!(
                "|{}>",
                states.iter().map(KetState::to_char).collect::<String>()
            )),
            PatAtomR::Unitary(term_r) => term_r.to_doc(),
        }
    }
}
