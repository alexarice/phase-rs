use std::ops::Range;

use winnow::{
    LocatingSlice, Parser, Result,
    ascii::{alphanumeric1, dec_uint, float, multispace0, multispace1},
    combinator::{alt, delimited, opt, preceded, repeat, separated, seq},
};

use crate::{
    command::Command,
    syntax::{
        KetState, Phase,
        raw::{AtomR, PatAtomR, PatTensorR, PatternR, TensorR, TermR},
    },
};

pub fn tm(input: &mut LocatingSlice<&str>) -> Result<TermR<Range<usize>>> {
    separated(1.., tensor, (multispace0, ';', multispace0))
        .with_span()
        .map(|(v, span)| TermR { terms: v, span })
        .parse_next(input)
}

fn tensor(input: &mut LocatingSlice<&str>) -> Result<TensorR<Range<usize>>> {
    separated(1.., atom, (multispace0, 'x', multispace0))
        .with_span()
        .map(|(v, span)| TensorR { terms: v, span })
        .parse_next(input)
}

fn phase(input: &mut LocatingSlice<&str>) -> Result<Phase> {
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

fn atom(input: &mut LocatingSlice<&str>) -> Result<AtomR<Range<usize>>> {
    (alt((
	delimited(("(", multispace0), tm, (multispace0, ")"))
	    .with_span()
	    .map(|(term, span)| AtomR::Brackets { term, span }),
	delimited(("sqrt(", multispace0), tm, (multispace0, ")"))
	    .with_span()
	    .map(|(inner, span)| AtomR::Sqrt { inner, span }),
	preceded("id", opt(dec_uint))
	    .with_span()
	    .map(|(qubits, span)| AtomR::Id { qubits: qubits.unwrap_or(1), span }),
	seq!(_: "if", _: multispace1, _: "let", _: multispace1, pattern, _: multispace1, _: "then", _: multispace1, atom)
	    .with_span()
	    .map(|((pattern, inner), span)| AtomR::IfLet{ pattern, inner: Box::new(inner), span }),
	phase.with_span().map(|(phase, span)| AtomR::Phase { phase, span }),
	"H".span().map(|span| AtomR::Hadamard { span }),
	identifier.with_span().map(|(name, span)| AtomR::Gate { name, span })
    )),
     opt((multispace0, "^", multispace0, "-1")))
	.with_span()
	.map(|((inner, invert), span)| {
	    if invert.is_some() {
		AtomR::Inverse { inner: Box::new(inner) , span }
	    } else {
		inner
	    }})
	.parse_next(input)
}

fn pattern(input: &mut LocatingSlice<&str>) -> Result<PatternR<Range<usize>>> {
    separated(1.., pattern_tensor, (multispace0, '.', multispace0))
        .with_span()
        .map(|(v, span)| PatternR { patterns: v, span })
        .parse_next(input)
}

fn pattern_tensor(input: &mut LocatingSlice<&str>) -> Result<PatTensorR<Range<usize>>> {
    separated(1.., pattern_atom, (multispace0, 'x', multispace0))
        .with_span()
        .map(|(v, span)| PatTensorR { patterns: v, span })
        .parse_next(input)
}

fn ket(input: &mut LocatingSlice<&str>) -> Result<PatAtomR<Range<usize>>> {
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

fn pattern_atom(input: &mut LocatingSlice<&str>) -> Result<PatAtomR<Range<usize>>> {
    alt((
        delimited(("(", multispace0), pattern, (multispace0, ")"))
            .with_span()
            .map(|(pattern, span)| PatAtomR::Brackets { pattern, span }),
        ket,
        tm.map(|x| PatAtomR::Unitary(Box::new(x))),
    ))
    .parse_next(input)
}

fn identifier(input: &mut LocatingSlice<&str>) -> Result<String> {
    alphanumeric1.map(|s: &str| s.to_owned()).parse_next(input)
}

fn gate(input: &mut LocatingSlice<&str>) -> Result<(String, TermR<Range<usize>>)> {
    seq!(_: ("gate", multispace1), identifier, _: (multispace0, "=", multispace0), tm, _: (multispace0, ",", multispace0)).parse_next(input)
}

pub fn command(input: &mut LocatingSlice<&str>) -> Result<Command<Range<usize>>> {
    let gates = repeat(0.., gate).parse_next(input)?;
    let term = tm.parse_next(input)?;
    Ok(Command { gates, term })
}
