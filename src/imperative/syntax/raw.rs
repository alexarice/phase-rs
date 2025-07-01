use pretty::RcDoc;

use crate::common::{KetState, Phase, Spanned, ToDoc};

pub type TypeR<S> = Spanned<S, TypeRInner>;
#[derive(Clone, Debug, PartialEq)]
pub struct TypeRInner(pub usize);

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
    Annotated {
        inner: Box<Pattern<S>>,
        ty: TypeR<S>,
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
