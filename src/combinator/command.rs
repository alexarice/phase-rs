//! A syntax of commands and their parsing.
//!
//! A `Command` is the top level structure accepted by the executable
//! They allow a sequence of gates to be defined before taking a term to evaluate.

use super::{
    typed_syntax::TermT,
    typecheck::{Env, TypeCheckError},
};
use crate::combinator::raw_syntax::TermR;

/// The Command structure: a runnable program.
#[derive(Clone, Debug)]
pub struct Command<S> {
    /// List of gates to define, with the name to bind them to.
    pub gates: Vec<(String, TermR<S>)>,
    /// Final term to evaluate.
    pub term: TermR<S>,
}

impl<S: Clone> Command<S> {
    /// Typecheck a command, building an `Env` with gate definitions.
    pub fn check(&self) -> Result<(Env, TermT), TypeCheckError<S>> {
        let mut env = Env::default();
        for (name, tm) in &self.gates {
            let t = tm.check(&env, None)?;
            env.0.insert(name.clone(), t);
        }
        let tm = self.term.check(&env, None)?;
        Ok((env, tm))
    }
}
