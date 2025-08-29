use std::iter::Sum;

use indexmap::IndexMap;

use crate::{
    ket::CompKetState,
    mvs::{common::Sliced, typecheck::TCResult},
    phase::Phase,
    text::Name,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetaId(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeT(pub usize, pub Vec<MetaId>);

impl Sum for TypeT {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut s = 0;
        let mut v = vec![];
        for x in iter {
            s += x.0;
            v.extend(x.1);
        }
        TypeT(s, v)
    }
}

pub struct TypeEnv(Vec<Option<TypeT>>);

impl TypeEnv {
    pub fn new_meta(&mut self) -> MetaId {
        self.0.push(None);
        MetaId(self.0.len())
    }
    fn resolve_index(&mut self, idx: MetaId) -> Option<TypeT> {
        if let Some(ty) = std::mem::take(&mut self.0[idx.0]) {
            let new_ty = self.resolve(&ty);
            self.0[idx.0] = Some(new_ty.clone());
            Some(new_ty)
        } else {
            None
        }
    }
    pub fn resolve(&mut self, ty: &TypeT) -> TypeT {
        let mut s = ty.0;
        let v =
            ty.1.iter()
                .flat_map(|&idx| {
                    if let Some(t) = self.resolve_index(idx) {
                        s += t.0;
                        t.1
                    } else {
                        vec![idx]
                    }
                })
                .collect();
        TypeT(s, v)
    }
    // pub fn unify(&mut self, ty1: &TypeT, ty2: &TypeT) -> TCResult<S, TypeT> {

    // }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitaryTypeT {
    pub args: IndexMap<Name, TypeT>,
    pub rootable: bool,
}

#[derive(Debug, Clone)]
pub enum UnitaryT {
    TopLevel(Name, Box<UnitaryT>),
    Def(UnitaryTypeT, Vec<UnitaryClauseT>),
    Inverse(Box<UnitaryT>),
    Sqrt(Box<UnitaryT>),
}

impl UnitaryT {
    pub fn get_type(&self) -> &UnitaryTypeT {
        match self {
            UnitaryT::TopLevel(_, unitary) => unitary.get_type(),
            UnitaryT::Def(unitary_type, _) => unitary_type,
            UnitaryT::Inverse(unitary) => unitary.get_type(),
            UnitaryT::Sqrt(unitary) => unitary.get_type(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnitaryClauseT {
    IfLet {
        pattern: Box<PatternT>,
        copattern: CopatternT,
        body: Vec<UnitaryClauseT>,
    },
    Phase(Phase),
    Call {
        unitary: Box<UnitaryT>,
        pos_args: Vec<CopatternT>,
        named_args: Vec<(Name, CopatternT)>,
    },
}

#[derive(Debug, Clone)]
pub enum CopatternT {
    Local(usize, TypeT, Sliced),
    Tensor(Vec<CopatternT>),
}

impl CopatternT {
    pub fn get_type(&self) -> TypeT {
        match self {
            CopatternT::Local(_, type_t, _) => type_t.clone(),
            CopatternT::Tensor(copattern_ts) => copattern_ts.iter().map(CopatternT::get_type).sum(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprT {
    Local(usize),
    Tensor(Vec<ExprT>),
    Ket(CompKetState),
    Ap(UnitaryT, Box<ExprT>),
}

#[derive(Debug, Clone)]
pub enum PatternClauseT {
    Let(Name, ExprT),
    Unitary(UnitaryClauseT),
}

#[derive(Debug, Clone)]
pub struct PatternT {
    pub clauses: Vec<PatternClauseT>,
    pub expr: ExprT,
}
