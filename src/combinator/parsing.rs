use std::ops::Range;

use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::{dec_uint, float, multispace0, multispace1, till_line_ending},
    combinator::{alt, cut_err, delimited, opt, preceded, repeat, separated, seq, terminated},
    error::{StrContext, StrContextValue},
};

use super::{
    command::Command,
    syntax::raw::{
        AtomR, AtomRInner, PatAtomR, PatAtomRInner, PatTensorR, PatTensorRInner, PatternR,
        PatternRInner, TensorR, TensorRInner, TermR, TermRInner,
    },
};
use crate::common::{KetState, Phase, Spanned, identifier, parse_spanned};

pub fn tm(input: &mut LocatingSlice<&str>) -> ModalResult<TermR<Range<usize>>> {
    parse_spanned(
        separated(1.., tensor, (multispace0, ';', multispace0))
            .context(StrContext::Label("term"))
            .map(|terms| TermRInner { terms }),
    )
    .parse_next(input)
}

fn tensor(input: &mut LocatingSlice<&str>) -> ModalResult<TensorR<Range<usize>>> {
    parse_spanned(
        separated(1.., atom, (multispace0, 'x', multispace0))
            .context(StrContext::Label("term"))
            .map(|terms| TensorRInner { terms }),
    )
    .parse_next(input)
}

fn phase(input: &mut LocatingSlice<&str>) -> ModalResult<Phase> {
    alt((
        "-1".value(Phase::MinusOne),
        "i".value(Phase::Imag),
        "-i".value(Phase::MinusImag),
        delimited(
            ("ph(", multispace0),
            float,
            (multispace0, "pi", multispace0, ")"),
        )
        .map(Phase::Angle),
    ))
    .parse_next(input)
}

fn atom(input: &mut LocatingSlice<&str>) -> ModalResult<AtomR<Range<usize>>> {
    parse_spanned((
	alt((
	    delimited(("(", multispace0),
		      cut_err(tm),
		      cut_err((multispace0, ")").context(StrContext::Expected(StrContextValue::CharLiteral(')')))))
		.map(|term| AtomRInner::Brackets { term }),
	    preceded(("sqrt", multispace0), cut_err(atom))
		.map(|inner| AtomRInner::Sqrt { inner: Box::new(inner) }),
	    preceded("id", opt(dec_uint))
		.map(|qubits| AtomRInner::Id { qubits: qubits.unwrap_or(1) }),
	    preceded("if", cut_err(seq!(_: multispace1, _: "let".context(StrContext::Expected(StrContextValue::StringLiteral("let"))), _: multispace1, pattern, _: multispace1, _: "then".context(StrContext::Expected(StrContextValue::StringLiteral("then"))), _: multispace1, atom)))
	    .map(|(pattern, inner)| AtomRInner::IfLet{ pattern, inner: Box::new(inner) }),
	    phase.map(|phase| AtomRInner::Phase { phase }),
	    identifier.map(|name| AtomRInner::Gate { name })
	)).context(StrContext::Expected(StrContextValue::CharLiteral('(')))
	    .context(StrContext::Expected(StrContextValue::StringLiteral("sqrt")))
	    .context(StrContext::Expected(StrContextValue::StringLiteral("id")))
	    .context(StrContext::Expected(StrContextValue::StringLiteral("if")))
	    .context(StrContext::Expected(StrContextValue::CharLiteral('H')))
	    .context(StrContext::Expected(StrContextValue::Description("identifier"))),
	opt((
	    multispace0,
	    "^",
	    multispace0,
	    cut_err("-1").context(StrContext::Expected(StrContextValue::StringLiteral("-1"))),
	)).context(StrContext::Label("term"))
    )
	.with_span()
	.map(|((inner, invert), span)| {
	    if invert.is_some() {
		AtomRInner::Inverse { inner: Box::new(Spanned { inner, span }) }
	    } else {
		inner
	    }}))
	.parse_next(input)
}

fn pattern(input: &mut LocatingSlice<&str>) -> ModalResult<PatternR<Range<usize>>> {
    parse_spanned(
        separated(1.., pattern_tensor, (multispace0, '.', multispace0))
            .map(|patterns| PatternRInner { patterns }),
    )
    .parse_next(input)
}

fn pattern_tensor(input: &mut LocatingSlice<&str>) -> ModalResult<PatTensorR<Range<usize>>> {
    parse_spanned(
        separated(1.., pattern_atom, (multispace0, 'x', multispace0))
            .map(|patterns| PatTensorRInner { patterns }),
    )
    .parse_next(input)
}

fn ket(input: &mut LocatingSlice<&str>) -> ModalResult<PatAtomRInner<Range<usize>>> {
    delimited(
        "|",
        repeat(
            1..,
            alt((
                "0".value(KetState::Zero),
                "1".value(KetState::One),
                "+".value(KetState::Plus),
                "-".value(KetState::Minus),
            )),
        ),
        ">",
    )
    .map(|states| PatAtomRInner::Ket { states })
    .parse_next(input)
}

fn pattern_atom(input: &mut LocatingSlice<&str>) -> ModalResult<PatAtomR<Range<usize>>> {
    parse_spanned(alt((
        delimited(("(", multispace0), pattern, (multispace0, ")"))
            .map(|pattern| PatAtomRInner::Brackets { pattern }),
        ket,
        tm.map(|x| PatAtomRInner::Unitary(Box::new(x))),
    )))
    .parse_next(input)
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

pub fn comment(input: &mut LocatingSlice<&str>) -> ModalResult<()> {
    (
        multispace0,
        repeat::<_, _, (), _, _>(0.., ("//", till_line_ending, multispace0).value(())),
    )
        .parse_next(input)?;
    Ok(())
}

pub fn command(input: &mut LocatingSlice<&str>) -> ModalResult<Command<Range<usize>>> {
    comment.parse_next(input)?;
    let gates = repeat(0.., terminated(gate, comment)).parse_next(input)?;
    let term = tm.context(StrContext::Label("Term")).parse_next(input)?;
    comment.parse_next(input)?;
    Ok(Command { gates, term })
}
