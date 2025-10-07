//! Normal-form syntax definitions
//!
//! Syntax for evaluated terms.
//! This is assumed to be typechecked/well-formed.

pub mod term;
pub use term::TermN;

pub mod pattern;
pub use pattern::PatternN;

use crate::{
    normal_syntax::term::AtomN,
    typed_syntax::{PatternType, TermType},
};

/// Trait for objects that can built with compositions, tensors, or from an `AtomN`.
pub trait Buildable {
    /// Build a composition object from a sequence of subobjects and a given type.
    /// Subobjects should be given in diagrammatic order, not function composition order.
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self;
    /// Build a tensor product from a sequence of subobjects.
    fn tensor(iter: impl Iterator<Item = Self>) -> Self;
    /// Build an object from an atom.
    fn atom(atom: AtomN) -> Self;
}

impl Buildable for TermN {
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self {
        TermN::Comp(iter.collect(), *ty)
    }

    fn tensor(iter: impl Iterator<Item = Self>) -> Self {
        TermN::Tensor(iter.collect())
    }

    fn atom(atom: AtomN) -> Self {
        TermN::Atom(atom)
    }
}

impl Buildable for PatternN {
    fn comp(iter: impl DoubleEndedIterator<Item = Self>, ty: &TermType) -> Self {
        PatternN::Comp(iter.rev().collect(), PatternType(ty.0, ty.0))
    }

    fn tensor(iter: impl Iterator<Item = Self>) -> Self {
        PatternN::Tensor(iter.collect())
    }

    fn atom(atom: AtomN) -> Self {
        PatternN::Unitary(Box::new(atom))
    }
}
