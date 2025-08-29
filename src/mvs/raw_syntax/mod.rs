//! Raw syntax definitions
//!
//! Raw syntax is used primarily for parsing and printed.
//! It is not assumed to be typechecked/well-formed.

use std::collections::HashMap;

use crate::{
    ket::CompKetState,
    mvs::{
        common::Sliced,
        typecheck::{Ctx, Env, Support, SupportAtom, TCResult, TypeCheckError},
        typed_syntax::{
            CopatternT, ExprT, PatternT, TypeT, UnitaryClauseT, UnitaryT, UnitaryTypeT,
        },
    },
    phase::Phase,
    text::{Name, Spanned},
};

pub type TypeR<S> = Spanned<S, TypeRInner>;
#[derive(Debug, Clone)]
pub enum TypeRInner {
    QReg(usize),
}

impl<S> TypeR<S> {
    pub fn check(&self) -> TypeT {
        match &self.inner {
            TypeRInner::QReg(n) => TypeT(*n, vec![]),
        }
    }
}

pub type ParamR<S> = Spanned<S, ParamRInner<S>>;
#[derive(Debug, Clone)]
pub struct ParamRInner<S> {
    pub name: Name,
    pub ty: TypeR<S>,
}

pub type UnitaryR<S> = Spanned<S, UnitaryRInner<S>>;
#[derive(Debug, Clone)]
pub enum UnitaryRInner<S> {
    TopLevel(Name),
    Def(Vec<ParamR<S>>, Vec<UnitaryClauseR<S>>),
    Inverse(Box<UnitaryR<S>>),
    Sqrt(Box<UnitaryR<S>>),
}

impl<S: Clone> UnitaryR<S> {
    pub fn check<'a>(
        &'a self,
        env: &'a Env,
        mut pos: impl ExactSizeIterator<Item = (&'a CopatternR<S>, TypeT)>,
        named: impl ExactSizeIterator<Item = (&'a Name, &'a CopatternR<S>, TypeT)>,
    ) -> TCResult<S, UnitaryT> {
        let unitary = self.infer(env)?;
        let unitary_type = unitary.get_type();
        // Check number of arguments first
        if unitary_type.args.len() != pos.len() + named.len() {
            return Err(Box::new(TypeCheckError::WrongNumberOfArgs {
                unitary: self.clone(),
                ty: unitary_type.clone(),
                expected_args: pos.len() + named.len(),
            }));
        }
        // Check named arguments
        let mut seen_names = HashMap::<&Name, &CopatternR<S>>::new();
        for (name, arg, ty) in named {
            if let Some(old_arg) = seen_names.insert(name, arg) {
                return Err(Box::new(TypeCheckError::UnitaryArgNamedTwice {
                    unitary: self.clone(),
                    name: name.clone(),
                    arg_1: old_arg.clone(),
                    arg_2: arg.clone(),
                }));
            }
            if let Some((idx, _, expected_ty)) = unitary_type.args.get_full(name) {
                if idx < pos.len() {
                    return Err(Box::new(TypeCheckError::UnitaryArgNamedAndPosition {
                        unitary: self.clone(),
                        pos_arg: pos.nth(idx).unwrap().0.clone(),
                        name: name.clone(),
                        named_arg: arg.clone(),
                    }));
                }
                if &ty != expected_ty {
                    return Err(Box::new(TypeCheckError::UnitaryArgTypeMismatch {
                        unitary: self.clone(),
                        argument: arg.clone(),
                        arg_type: ty,
                        expected_type: expected_ty.clone(),
                    }));
                }
            } else {
                return Err(Box::new(TypeCheckError::UnitaryUnknownNamedArg {
                    unitary: self.clone(),
                    argument: arg.clone(),
                    name: name.clone(),
                }));
            }
        }
        // Now check positional arguments
        for ((arg, ty), expected_ty) in pos.zip(unitary_type.args.values()) {
            if &ty != expected_ty {
                return Err(Box::new(TypeCheckError::UnitaryArgTypeMismatch {
                    unitary: self.clone(),
                    argument: arg.clone(),
                    arg_type: ty,
                    expected_type: expected_ty.clone(),
                }));
            }
        }
        Ok(unitary)
    }

    pub fn infer(&self, env: &Env) -> TCResult<S, UnitaryT> {
        match &self.inner {
            UnitaryRInner::TopLevel(name) => {
                if let Some(t) = env.0.get(name) {
                    Ok(UnitaryT::TopLevel(name.clone(), Box::new(t.clone())))
                } else {
                    Err(Box::new(TypeCheckError::UnknownSymbol {
                        name: name.clone(),
                        span: self.span.clone(),
                    }))
                }
            }
            UnitaryRInner::Def(parameters, clauses) => {
                let ctx = Ctx(parameters
                    .iter()
                    .map(|p| (p.inner.name.clone(), p.inner.ty.check()))
                    .collect());
                let checked_clauses = clauses
                    .iter()
                    .map(|c| UnitaryClauseR::check(c, env, &ctx))
                    .collect::<Result<Vec<_>, _>>()?;
                let ty = UnitaryTypeT {
                    args: ctx.0,
                    rootable: checked_clauses.len() <= 1,
                };
                Ok(UnitaryT::Def(ty, checked_clauses))
            }
            UnitaryRInner::Inverse(unitary) => Ok(UnitaryT::Inverse(Box::new(unitary.infer(env)?))),
            UnitaryRInner::Sqrt(unitary) => {
                let checked = unitary.infer(env)?;
                if checked.get_type().rootable {
                    Ok(UnitaryT::Sqrt(Box::new(checked)))
                } else {
                    Err(Box::new(TypeCheckError::TermNotRootable {
                        unitary: unitary.as_ref().clone(),
                    }))
                }
            }
        }
    }
}

pub type UnitaryClauseR<S> = Spanned<S, UnitaryClauseRInner<S>>;
#[derive(Debug, Clone)]
pub enum UnitaryClauseRInner<S> {
    IfLet {
        pattern: Box<PatternR<S>>,
        copattern: CopatternR<S>,
        body: Vec<UnitaryClauseR<S>>,
    },
    Phase(Phase),
    Call {
        unitary: Box<UnitaryR<S>>,
        pos_args: Vec<CopatternR<S>>,
        named_args: Vec<(Name, CopatternR<S>)>,
    },
}

impl<S: Clone> UnitaryClauseR<S> {
    pub fn check(&self, env: &Env, ctx: &Ctx) -> TCResult<S, UnitaryClauseT> {
        match &self.inner {
            UnitaryClauseRInner::IfLet {
                pattern,
                copattern,
                body,
            } => todo!(),
            UnitaryClauseRInner::Phase(phase) => Ok(UnitaryClauseT::Phase(*phase)),
            UnitaryClauseRInner::Call {
                unitary,
                pos_args,
                named_args,
            } => {
                let mut support = Support::<CopatternR<S>>::default();
                let checked_pos_args: Vec<CopatternT> = pos_args
                    .iter()
                    .map(|arg| arg.infer(ctx, &mut support))
                    .collect::<Result<Vec<_>, _>>()?;

                let checked_named_args: Vec<(Name, CopatternT)> = named_args
                    .iter()
                    .map(|(name, arg)| Ok((name.clone(), arg.infer(ctx, &mut support)?)))
                    .collect::<TCResult<S, Vec<_>>>()?;

                let checked_unitary = unitary.check(
                    env,
                    pos_args
                        .iter()
                        .zip(&checked_pos_args)
                        .map(|(copattern, checked_copattern)| {
                            (copattern, checked_copattern.get_type())
                        }),
                    named_args.iter().zip(&checked_named_args).map(
                        |((name, copattern), (_, checked_copattern))| {
                            (name, copattern, checked_copattern.get_type())
                        },
                    ),
                )?;

                Ok(UnitaryClauseT::Call {
                    unitary: Box::new(checked_unitary),
                    pos_args: checked_pos_args,
                    named_args: checked_named_args,
                })
            }
        }
    }
}

pub type CopatternR<S> = Spanned<S, CopatternRInner<S>>;
#[derive(Debug, Clone)]
pub enum CopatternRInner<S> {
    Local(Name, Sliced),
    Tensor(Vec<CopatternR<S>>),
}

impl<S: Clone> CopatternR<S> {
    pub fn infer(
        &self,
        ctx: &Ctx,
        support: &mut Support<CopatternR<S>>,
    ) -> TCResult<S, CopatternT> {
        match &self.inner {
            CopatternRInner::Local(name, sliced) => {
                if let Some((var, _, ty)) = ctx.0.get_full(name) {
                    let atom = SupportAtom {
                        var,
                        range: sliced.to_range(),
                    };
                    if let Some(copattern_1) = support.get_clash(&atom) {
                        return Err(Box::new(TypeCheckError::CopatternSupportClash {
                            copattern_1: copattern_1.clone(),
                            copattern_2: self.clone(),
                            name: name.clone(),
                        }));
                    }
                    support.insert(atom, self.clone());
                    Ok(CopatternT::Local(var, ty.clone(), sliced.clone()))
                } else {
                    Err(Box::new(TypeCheckError::UnknownSymbol {
                        name: name.clone(),
                        span: self.span.clone(),
                    }))
                }
            }
            CopatternRInner::Tensor(copatterns) => {
                let checked = copatterns
                    .iter()
                    .map(|c| {
                        let checked = c.infer(ctx, support)?;
                        Ok(checked)
                    })
                    .collect::<TCResult<S, Vec<_>>>()?;

                Ok(CopatternT::Tensor(checked))
            }
        }
    }
}

pub type ExprR<S> = Spanned<S, ExprRInner<S>>;
#[derive(Debug, Clone)]
pub enum ExprRInner<S> {
    Local(Name, Option<TypeR<S>>),
    Tensor(Vec<ExprR<S>>),
    Ket(CompKetState),
    Ap(UnitaryR<S>, Box<ExprR<S>>),
}

impl<S> ExprR<S> {
    pub fn check(&self, env: &Env) -> TCResult<S, ExprT> {
        match &self.inner {
            ExprRInner::Local(name, ty) => todo!(),
            ExprRInner::Tensor(spanneds) => todo!(),
            ExprRInner::Ket(comp_ket_state) => todo!(),
            ExprRInner::Ap(spanned, spanned1) => todo!(),
        }
    }

    pub fn infer(&self, env: &Env, ctx: &mut Ctx) -> TCResult<S, ExprT> {
        match &self.inner {
            ExprRInner::Local(name, ty) => todo!(),
            ExprRInner::Tensor(spanneds) => todo!(),
            ExprRInner::Ket(comp_ket_state) => todo!(),
            ExprRInner::Ap(spanned, spanned1) => todo!(),
        }
    }
}

pub type PatternClauseR<S> = Spanned<S, PatternClauseRInner<S>>;
#[derive(Debug, Clone)]
pub enum PatternClauseRInner<S> {
    Let(Name, ExprR<S>),
    Unitary(UnitaryClauseR<S>),
}

pub type PatternR<S> = Spanned<S, PatternRInner<S>>;
#[derive(Debug, Clone)]
pub struct PatternRInner<S> {
    pub clauses: Vec<PatternClauseR<S>>,
    pub expr: ExprR<S>,
}

impl<S> PatternR<S> {
    pub fn check(&self, env: &Env, ret_type: TypeT) -> TCResult<S, PatternT> {
        todo!()
    }
}
