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

use crate::{
    raw_syntax::TermR,
    text::{HasParser, Name, Span, comment_parser},
    typecheck::{Env, TypeCheckError},
    typed_syntax::TermT,
};

/// The Command structure: a runnable program.
#[derive(Clone, Debug)]
pub struct Command<S> {
    /// List of gates to define, with the name to bind them to.
    pub gates: Vec<(Name, TermR<S>)>,
    /// Final term to evaluate.
    pub term: TermR<S>,
}

impl<S: Span> Command<S> {
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

impl HasParser for Command<Range<usize>> {
    fn parser(input: &mut LocatingSlice<&str>) -> ModalResult<Self> {
        let gate = preceded(
	"gate",
	cut_err(seq!(_: multispace1,
		     Name::parser,
		     _: (multispace0, "=", multispace0).context(StrContext::Expected(StrContextValue::CharLiteral('='))),
		     TermR::parser,
		     _: (multispace0, ","))).context(StrContext::Label("gate definition"))
	);

        comment_parser.parse_next(input)?;
        let gates = repeat(0.., terminated(gate, comment_parser)).parse_next(input)?;
        let term = TermR::parser
            .context(StrContext::Label("Term"))
            .parse_next(input)?;
        comment_parser.parse_next(input)?;
        Ok(Command { gates, term })
    }
}
