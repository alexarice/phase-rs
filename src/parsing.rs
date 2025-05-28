use std::ops::Range;

use winnow::LocatingSlice;
use winnow::Parser;
use winnow::Result;
use winnow::ascii::{dec_uint, float, multispace0, multispace1};
use winnow::combinator::separated;
use winnow::combinator::{alt, delimited, preceded, repeat, seq};
use winnow::error::StrContextValue;

use crate::syntax::KetState;
use crate::syntax::raw::{PatternR, TermR, TypeR};

pub fn qreg(input: &mut LocatingSlice<&str>) -> Result<usize> {
    preceded('q', dec_uint).parse_next(input)
}

pub fn ty(input: &mut LocatingSlice<&str>) -> Result<TypeR<Range<usize>>> {
    seq!(qreg, _: multispace0, _: "<->", _: multispace0, qreg)
        .with_span()
        .verify_map(|((before, after), span)| {
            if before == after {
                Some(TypeR::Unitary(before, span))
            } else {
                None
            }
        })
        .context(winnow::error::StrContext::Label("unitary type"))
        .context(winnow::error::StrContext::Expected(
            StrContextValue::Description("source and target to be identical"),
        ))
        .parse_next(input)
}

pub fn tm(input: &mut LocatingSlice<&str>) -> Result<TermR<Range<usize>>> {
    separated(1.., tensor, (multispace0, ';', multispace0))
        .with_span()
        .map(|(v, span): (Vec<_>, _)| {
            if v.len() == 1 {
                v.into_iter().next().unwrap()
            } else {
                TermR::Comp { terms: v, span }
            }
        })
        .parse_next(input)
}

fn tensor(input: &mut LocatingSlice<&str>) -> Result<TermR<Range<usize>>> {
    separated(1.., atom, (multispace0, 'x', multispace0))
        .with_span()
        .map(|(v, span): (Vec<_>, _)| {
            if v.len() == 1 {
                v.into_iter().next().unwrap()
            } else {
                TermR::Tensor { terms: v, span }
            }
        })
        .parse_next(input)
}

fn atom(input: &mut LocatingSlice<&str>) -> Result<TermR<Range<usize>>> {
    alt((
	delimited(("(", multispace0), tm, (multispace0, ")")),
	preceded("id", dec_uint).with_span().map(|(qubits, span)| TermR::Id { qubits, span }),
	delimited(("ph(", multispace0), float, (multispace0, "pi", multispace0, ")")).with_span().map(|(angle, span)| TermR::Phase { angle, span }),
	"H".span().map(|span| TermR::Hadamard { span }),
	seq!(_: "if", _: multispace1, _: "let", _: multispace1, pattern, _: multispace1, _: "then", _: multispace1, atom).with_span().map(|((pattern, inner), span)| TermR::IfLet{ pattern, inner: Box::new(inner), span })
    )).parse_next(input)
}

fn pattern(input: &mut LocatingSlice<&str>) -> Result<PatternR<Range<usize>>> {
    separated(1.., pattern_tensor, (multispace0, '.', multispace0))
        .with_span()
        .map(|(v, span): (Vec<_>, _)| {
            if v.len() == 1 {
                v.into_iter().next().unwrap()
            } else {
                PatternR::Comp { patterns: v, span }
            }
        })
        .parse_next(input)
}

fn pattern_tensor(input: &mut LocatingSlice<&str>) -> Result<PatternR<Range<usize>>> {
    separated(1.., pattern_atom, (multispace0, 'x', multispace0))
        .with_span()
        .map(|(v, span): (Vec<_>, _)| {
            if v.len() == 1 {
                v.into_iter().next().unwrap()
            } else {
                PatternR::Tensor { patterns: v, span }
            }
        })
        .parse_next(input)
}

fn ket(input: &mut LocatingSlice<&str>) -> Result<PatternR<Range<usize>>> {
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
    .map(|(states, span)| PatternR::Ket { states, span })
    .parse_next(input)
}

fn pattern_atom(input: &mut LocatingSlice<&str>) -> Result<PatternR<Range<usize>>> {
    alt((
        delimited(("(", multispace0), pattern, (multispace0, ")")),
        ket,
        tm.map(|x| PatternR::Unitary(Box::new(x))),
    ))
    .parse_next(input)
}
