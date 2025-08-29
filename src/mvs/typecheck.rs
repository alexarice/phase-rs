use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    ops::Range,
};

use indexmap::IndexMap;

use crate::{
    mvs::{
        raw_syntax::{CopatternR, UnitaryR},
        typed_syntax::{TypeT, UnitaryT, UnitaryTypeT},
    },
    text::Name,
};

/// Errors that can occur during typechecking.
#[derive(Debug, Clone)]
pub enum TypeCheckError<S> {
    /// An unknown top-level symbol.
    UnknownSymbol {
        /// The unknown symbol encountered
        name: Name,
        /// Span of symbol
        span: S,
    },
    /// Square root applied to a non-rootable term.
    TermNotRootable {
        /// The square root term.
        unitary: UnitaryR<S>,
    },
    /// Wrong number of arguments given to unitary
    WrongNumberOfArgs {
        /// Unitary term
        unitary: UnitaryR<S>,
        ty: UnitaryTypeT,
        expected_args: usize,
    },
    /// Argument to unitary's type does not match expected type
    UnitaryArgTypeMismatch {
        /// Unitary term
        unitary: UnitaryR<S>,
        /// Argument
        argument: CopatternR<S>,
        /// Argument type
        arg_type: TypeT,
        /// Expected type
        expected_type: TypeT,
    },
    /// Named argument has unknown name
    UnitaryUnknownNamedArg {
        /// Unitary term
        unitary: UnitaryR<S>,
        /// Argument
        argument: CopatternR<S>,
        /// Name of argument
        name: Name,
    },
    /// Argument is given both by position and by name
    UnitaryArgNamedAndPosition {
        /// Unitary term
        unitary: UnitaryR<S>,
        /// Position argument
        pos_arg: CopatternR<S>,
        /// Name
        name: Name,
        /// Named argument
        named_arg: CopatternR<S>,
    },
    /// Clash of support in Copattern
    CopatternSupportClash {
        /// Subterm 1
        copattern_1: CopatternR<S>,
        /// Subterm 2
        copattern_2: CopatternR<S>,
        /// Name of resued variable
        name: Name,
    },
    /// Argument is given by two different named arguments.
    UnitaryArgNamedTwice {
        unitary: UnitaryR<S>,
        /// Name
        name: Name,
        /// Argument 1,
        arg_1: CopatternR<S>,
        /// Argument 2,
        arg_2: CopatternR<S>,
    },
}

pub type TCResult<S, T> = Result<T, Box<TypeCheckError<S>>>;

/// Typing environment, holding definitions of top level symbols.
#[derive(Default)]
pub struct Env(pub(crate) HashMap<Name, UnitaryT>);

/// Typing context, holding types of local symbols.
pub struct Ctx(pub(crate) IndexMap<Name, TypeT>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportAtom {
    pub(crate) var: usize,
    pub(crate) range: Option<Range<usize>>,
}

impl PartialOrd for SupportAtom {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SupportAtom {
    fn cmp(&self, other: &Self) -> Ordering {
        let fst = self.var.cmp(&other.var);
        if fst.is_ne() {
            return fst;
        }
        match (&self.range, &other.range) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(lhs), Some(rhs)) => lhs.start.cmp(&rhs.start).then(lhs.end.cmp(&rhs.end)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Support<T>(BTreeMap<SupportAtom, T>);

impl<T> Default for Support<T> {
    fn default() -> Self {
        Support(BTreeMap::default())
    }
}

impl<T> IntoIterator for Support<T> {
    type Item = (SupportAtom, T);

    type IntoIter = <BTreeMap<SupportAtom, T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn cmp_range(lhs: &Option<Range<usize>>, rhs: &Option<Range<usize>>) -> Ordering {
    match (lhs, rhs) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(r1), Some(r2)) => r1.start.cmp(&r2.start).then(r1.end.cmp(&r2.end)),
    }
}

impl<T> Support<T> {
    pub fn insert(&mut self, key: SupportAtom, value: T) {
        self.0.insert(key, value);
    }

    pub fn new(key: SupportAtom, value: T) -> Self {
        Support(BTreeMap::from([(key, value)]))
    }

    pub fn get_clash(&self, key: &SupportAtom) -> Option<&T> {
        let lower_bound = SupportAtom {
            var: key.var,
            range: None,
        };
        let upper_bound = SupportAtom {
            var: key.var,
            range: None,
        };
        let before = self.0.range(&lower_bound..key).next_back();
        let after = self.0.range(key..&upper_bound).next();
        before
            .and_then(|(x, t)| {
                if x.var != key.var || cmp_range(&x.range, &key.range).is_lt() {
                    None
                } else {
                    Some(t)
                }
            })
            .or(after.and_then(|(x, t)| {
                if x.var != key.var || cmp_range(&key.range, &x.range).is_lt() {
                    None
                } else {
                    Some(t)
                }
            }))
    }
}
