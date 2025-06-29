use std::ops::Range;

use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::{alphanumeric1, dec_uint, float, multispace0, multispace1},
    combinator::{alt, cut_err, delimited, opt, preceded, repeat, separated, seq, terminated},
    error::{StrContext, StrContextValue},
    token::take_until,
};

use super::{
    command::Command,
    syntax::raw::{AtomR, PatAtomR, PatTensorR, PatternR, TensorR, TermR},
};
use crate::common::{KetState, Phase};

pub fn tm(input: &mut LocatingSlice<&str>) -> ModalResult<TermR<Range<usize>>> {
    separated(1.., tensor, (multispace0, ';', multispace0))
        .context(StrContext::Label("term"))
        .with_span()
        .map(|(v, span)| TermR { terms: v, span })
        .parse_next(input)
}

fn tensor(input: &mut LocatingSlice<&str>) -> ModalResult<TensorR<Range<usize>>> {
    separated(1.., atom, (multispace0, 'x', multispace0))
        .context(StrContext::Label("term"))
        .with_span()
        .map(|(v, span)| TensorR { terms: v, span })
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
    (alt((
	delimited(("(", multispace0),
		  cut_err(tm),
		  cut_err((multispace0, ")").context(StrContext::Expected(StrContextValue::CharLiteral(')')))))
	    .with_span()
	    .map(|(term, span)| AtomR::Brackets { term, span }),
	preceded(("sqrt", multispace0), cut_err(atom))
	    .with_span()
	    .map(|(inner, span)| AtomR::Sqrt { inner: Box::new(inner), span }),
	preceded("id", opt(dec_uint))
	    .with_span()
	    .map(|(qubits, span)| AtomR::Id { qubits: qubits.unwrap_or(1), span }),
	preceded("if", cut_err(seq!(_: multispace1, _: "let".context(StrContext::Expected(StrContextValue::StringLiteral("let"))), _: multispace1, pattern, _: multispace1, _: "then".context(StrContext::Expected(StrContextValue::StringLiteral("then"))), _: multispace1, atom)))
	    .with_span()
	    .map(|((pattern, inner), span)| AtomR::IfLet{ pattern, inner: Box::new(inner), span }),
	phase.with_span().map(|(phase, span)| AtomR::Phase { phase, span }),
	identifier.with_span().map(|(name, span)| AtomR::Gate { name, span })
    )).context(StrContext::Expected(StrContextValue::CharLiteral('(')))
     .context(StrContext::Expected(StrContextValue::StringLiteral("sqrt")))
     .context(StrContext::Expected(StrContextValue::StringLiteral("id")))
     .context(StrContext::Expected(StrContextValue::StringLiteral("if")))
     .context(StrContext::Expected(StrContextValue::CharLiteral('H')))
     .context(StrContext::Expected(StrContextValue::Description("identifier"))),
     opt((multispace0, "^", multispace0, cut_err("-1").context(StrContext::Expected(StrContextValue::StringLiteral("-1")))))).context(StrContext::Label("term"))
	.with_span()
	.map(|((inner, invert), span)| {
	    if invert.is_some() {
		AtomR::Inverse { inner: Box::new(inner) , span }
	    } else {
		inner
	    }})
	.parse_next(input)
}

fn pattern(input: &mut LocatingSlice<&str>) -> ModalResult<PatternR<Range<usize>>> {
    separated(1.., pattern_tensor, (multispace0, '.', multispace0))
        .with_span()
        .map(|(v, span)| PatternR { patterns: v, span })
        .parse_next(input)
}

fn pattern_tensor(input: &mut LocatingSlice<&str>) -> ModalResult<PatTensorR<Range<usize>>> {
    separated(1.., pattern_atom, (multispace0, 'x', multispace0))
        .with_span()
        .map(|(v, span)| PatTensorR { patterns: v, span })
        .parse_next(input)
}

fn ket(input: &mut LocatingSlice<&str>) -> ModalResult<PatAtomR<Range<usize>>> {
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
    .with_span()
    .map(|(states, span)| PatAtomR::Ket { states, span })
    .parse_next(input)
}

fn pattern_atom(input: &mut LocatingSlice<&str>) -> ModalResult<PatAtomR<Range<usize>>> {
    alt((
        delimited(("(", multispace0), pattern, (multispace0, ")"))
            .with_span()
            .map(|(pattern, span)| PatAtomR::Brackets { pattern, span }),
        ket,
        tm.map(|x| PatAtomR::Unitary(Box::new(x))),
    ))
    .parse_next(input)
}

fn identifier(input: &mut LocatingSlice<&str>) -> ModalResult<String> {
    alphanumeric1
        .map(|s: &str| s.to_owned())
        .context(StrContext::Label("identifier"))
        .context(StrContext::Expected(StrContextValue::Description(
            "alphanumeric string",
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
        repeat::<_, _, (), _, _>(0.., ("//", take_until(0.., "\n"), multispace0).value(())),
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
