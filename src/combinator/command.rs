//! A syntax of commands and their parsing.
//!
//! A `Command` is the top level structure accepted by the executable
//! They allow a sequence of gates to be defined before taking a term to evaluate.

use std::ops::Range;

use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::{multispace0, multispace1},
    combinator::{cut_err, preceded, repeat, seq, terminated},
    error::{StrContext, StrContextValue},
};

use super::{
    typecheck::{Env, TypeCheckError},
    typed_syntax::TermT,
};
use crate::{
    combinator::{parsing::tm, raw_syntax::TermR},
    text::{comment, identifier},
};

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

fn gate(input: &mut LocatingSlice<&str>) -> ModalResult<(String, TermR<Range<usize>>)> {
    preceded(
	"gate",
	cut_err(seq!(_: multispace1,
		     identifier,
		     _: (multispace0, "=", multispace0).context(StrContext::Expected(StrContextValue::CharLiteral('='))),
		     tm,
		     _: (multispace0, ","))).context(StrContext::Label("gate definition"))
    ).parse_next(input)
}

/// Parse a command
pub fn command(input: &mut LocatingSlice<&str>) -> ModalResult<Command<Range<usize>>> {
    comment.parse_next(input)?;
    let gates = repeat(0.., terminated(gate, comment)).parse_next(input)?;
    let term = tm.context(StrContext::Label("Term")).parse_next(input)?;
    comment.parse_next(input)?;
    Ok(Command { gates, term })
}
