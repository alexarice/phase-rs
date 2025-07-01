use pretty::RcDoc;

use crate::common::{KetState, Phase, Spanned, ToDoc};

pub type TypeR<S> = Spanned<S, TypeRInner>;
#[derive(Clone, Debug, PartialEq)]
pub struct TypeRInner(usize);

pub type Copattern<S> = Spanned<S, CopatternInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub enum CopatternInner<S> {
    Var {
        name: String,
    },
    Annotated {
        inner: Box<Copattern<S>>,
        ty: TypeR<S>,
    },
    Tensor {
        copatterns: Vec<Copattern<S>>,
    },
}

pub type Pattern<S> = Spanned<S, PatternInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub enum PatternInner<S> {
    Var {
        name: String,
    },
    Tensor {
        patterns: Vec<Pattern<S>>,
        span: S,
    },
    Comp {
        unitary: Box<Unitary<S>>,
        pattern: Box<Pattern<S>>,
        span: S,
    },
    Ket {
        states: Vec<KetState>,
        span: S,
    },
}

pub type Unitary<S> = Spanned<S, UnitaryInner<S>>;
#[derive(Clone, Debug, PartialEq)]
pub enum UnitaryInner<S> {
    Id,
    Comp {
        unitaries: Vec<Unitary<S>>,
    },
    Phase {
        phase: Phase,
    },
    IfLet {
        pattern: Pattern<S>,
        copattern: Copattern<S>,
        inner: Box<Unitary<S>>,
    },
    Gate {
        name: String,
    },
    Inverse {
        inner: Box<Unitary<S>>,
    },
    Sqrt {
        inner: Box<Unitary<S>>,
    },
}

impl ToDoc for TypeRInner {
    fn to_doc(&self) -> RcDoc {
        todo!()
    }
}

// impl<S> AtomR<S> {
//     pub fn to_doc(&self) -> RcDoc {
//         match self {
//             AtomR::Brackets { term, .. } => RcDoc::text("(")
//                 .append(RcDoc::line().append(term.to_doc()).nest(2))
//                 .append(RcDoc::line())
//                 .append(")")
//                 .group(),
//             AtomR::Id { qubits, .. } => RcDoc::text(if *qubits == 1 {
//                 Cow::Borrowed("id")
//             } else {
//                 Cow::Owned(format!("id{qubits}"))
//             }),
//             AtomR::Phase { phase, .. } => phase.to_doc(),
//             AtomR::IfLet { pattern, inner, .. } => RcDoc::text("if let")
//                 .append(RcDoc::line().append(pattern.to_doc()).nest(2))
//                 .append(RcDoc::line())
//                 .append("then")
//                 .group()
//                 .append(RcDoc::line().append(inner.to_doc()).nest(2))
//                 .group(),
//             AtomR::Gate { name, .. } => RcDoc::text(name),
//             AtomR::Inverse { inner, .. } => inner.to_doc().append(" ^ -1"),
//             AtomR::Sqrt { inner, .. } => RcDoc::text("sqrt(")
//                 .append(RcDoc::line().append(inner.to_doc()).nest(2))
//                 .append(RcDoc::line())
//                 .append(")")
//                 .group(),
//         }
//     }
// }

// impl<S> PatternR<S> {
//     pub fn to_doc(&self) -> RcDoc {
//         RcDoc::intersperse(
//             self.patterns.iter().map(PatTensorR::to_doc),
//             RcDoc::line().append(". "),
//         )
//         .group()
//     }
// }

// impl<S> PatTensorR<S> {
//     pub fn to_doc(&self) -> RcDoc {
//         RcDoc::intersperse(
//             self.patterns.iter().map(PatAtomR::to_doc),
//             RcDoc::line().append("x "),
//         )
//         .group()
//     }
// }

// impl<S> PatAtomR<S> {
//     pub fn to_doc(&self) -> RcDoc {
//         match self {
//             PatAtomR::Brackets { pattern, .. } => RcDoc::text("(")
//                 .append(RcDoc::line().append(pattern.to_doc()).nest(2))
//                 .append(RcDoc::line())
//                 .append(")")
//                 .group(),
//             PatAtomR::Ket { states, .. } => RcDoc::text(format!(
//                 "|{}>",
//                 states.iter().map(KetState::to_char).collect::<String>()
//             )),
//             PatAtomR::Unitary(term_r) => term_r.to_doc(),
//         }
//     }
// }
