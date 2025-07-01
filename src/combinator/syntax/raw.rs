use std::borrow::Cow;

use pretty::RcDoc;

use crate::common::{KetState, Phase, Spanned, ToDoc};

pub type TermR<S> = Spanned<S, TermRInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub struct TermRInner<S> {
    pub terms: Vec<TensorR<S>>,
}

pub type TensorR<S> = Spanned<S, TensorRInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub struct TensorRInner<S> {
    pub terms: Vec<AtomR<S>>,
}

pub type AtomR<S> = Spanned<S, AtomRInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub enum AtomRInner<S> {
    Brackets {
        term: TermR<S>,
    },
    Id {
        qubits: usize,
    },
    Phase {
        phase: Phase,
    },
    IfLet {
        pattern: PatternR<S>,
        inner: Box<AtomR<S>>,
    },
    Gate {
        name: String,
    },
    Inverse {
        inner: Box<AtomR<S>>,
    },
    Sqrt {
        inner: Box<AtomR<S>>,
    },
}

pub type PatternR<S> = Spanned<S, PatternRInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub struct PatternRInner<S> {
    pub patterns: Vec<PatTensorR<S>>,
}

pub type PatTensorR<S> = Spanned<S, PatTensorRInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub struct PatTensorRInner<S> {
    pub patterns: Vec<PatAtomR<S>>,
}

pub type PatAtomR<S> = Spanned<S, PatAtomRInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub enum PatAtomRInner<S> {
    Brackets { pattern: PatternR<S> },
    Ket { states: Vec<KetState> },
    Unitary(Box<TermR<S>>),
}

impl<S> ToDoc for TermRInner<S> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.terms.iter().map(TensorR::to_doc),
            RcDoc::text(";").append(RcDoc::line()),
        )
        .group()
    }
}

impl<S> ToDoc for TensorRInner<S> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.terms.iter().map(AtomR::to_doc),
            RcDoc::line().append("x "),
        )
        .group()
    }
}

impl<S> ToDoc for AtomRInner<S> {
    fn to_doc(&self) -> RcDoc {
        match self {
            AtomRInner::Brackets { term, .. } => RcDoc::text("(")
                .append(RcDoc::line().append(term.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
            AtomRInner::Id { qubits, .. } => RcDoc::text(if *qubits == 1 {
                Cow::Borrowed("id")
            } else {
                Cow::Owned(format!("id{qubits}"))
            }),
            AtomRInner::Phase { phase, .. } => phase.to_doc(),
            AtomRInner::IfLet { pattern, inner, .. } => RcDoc::text("if let")
                .append(RcDoc::line().append(pattern.to_doc()).nest(2))
                .append(RcDoc::line())
                .append("then")
                .group()
                .append(RcDoc::line().append(inner.to_doc()).nest(2))
                .group(),
            AtomRInner::Gate { name, .. } => RcDoc::text(name),
            AtomRInner::Inverse { inner, .. } => inner.to_doc().append(" ^ -1"),
            AtomRInner::Sqrt { inner, .. } => RcDoc::text("sqrt(")
                .append(RcDoc::line().append(inner.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
        }
    }
}

impl<S> ToDoc for PatternRInner<S> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.patterns.iter().map(PatTensorR::to_doc),
            RcDoc::line().append(". "),
        )
        .group()
    }
}

impl<S> ToDoc for PatTensorRInner<S> {
    fn to_doc(&self) -> RcDoc {
        RcDoc::intersperse(
            self.patterns.iter().map(PatAtomR::to_doc),
            RcDoc::line().append("x "),
        )
        .group()
    }
}

impl<S> ToDoc for PatAtomRInner<S> {
    fn to_doc(&self) -> RcDoc {
        match self {
            PatAtomRInner::Brackets { pattern, .. } => RcDoc::text("(")
                .append(RcDoc::line().append(pattern.to_doc()).nest(2))
                .append(RcDoc::line())
                .append(")")
                .group(),
            PatAtomRInner::Ket { states, .. } => RcDoc::text(format!(
                "|{}>",
                states.iter().map(KetState::to_char).collect::<String>()
            )),
            PatAtomRInner::Unitary(term_r) => term_r.to_doc(),
        }
    }
}
