//! Raw syntax definitions
//!
//! Raw syntax is used primarily for parsing and printed.
//! It is not assumed to be typechecked/well-formed.

use crate::{ket::CompKetState, phase::Phase, text::{Name, Spanned}};

pub type TypeR<S> = Spanned<S, TypeRInner<S>>;

pub enum TypeRInner<S> {
    Bool,
    QReg(usize),
    Func {
	unitary: bool,
	params: Vec<ParamR<S>>,
	ret: Box<TypeR<S>>,
    },
    Tuple(Vec<TypeR<S>>),
}

pub type ParamR<S> = Spanned<S, ParamRInner<S>>;
pub struct ParamRInner<S> {
    pub name: Name,
    pub inout: bool,
    pub ty: TypeR<S>,
}


pub type ExprR<S> = Spanned<S, ExprRInner<S>>;

pub enum ExprRInner<S> {
    MeasureCompBasis(Box<ExprR<S>>),
    FunCall(Box<ExprR<S>>, Vec<ExprR<S>>, Vec<(Name, ExprR<S>)>),
    Var(Name),
    Ket(CompKetState),
    Tensor(Box<ExprR<S>>, Box<ExprR<S>>),
    Ap(Box<ExprR<S>>, Box<ExprR<S>>),
}

pub type ClauseR<S> = Spanned<S, ClauseRInner<S>>;

pub enum ClauseRInner<S> {
    Let(Vec<Name>, Option<TypeR<S>>, ExprR<S>),
    Mutate(Vec<Name>, ExprR<S>),
    Expr(ExprR<S>),
    Phase(Phase),
    IfLet {
	pattern: ExprR<S>,
	copattern: ExprR<S>,
	body: TermR<S>,
    },
}

pub type TermR<S> = Spanned<S, TermRInner<S>>;

pub struct TermRInner<S>(Vec<ClauseR<S>>);
