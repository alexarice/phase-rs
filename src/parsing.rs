use std::ops::Range;

use winnow::LocatingSlice;
use winnow::Parser;
use winnow::Result;
use winnow::ascii::{dec_uint, float, multispace0, multispace1};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, seq};
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
    (tensor, opt(preceded((multispace0, ';', multispace0), tm)))
        .with_span()
        .map(|((t1, rest), span)| {
            if let Some(t2) = rest {
                TermR::Comp {
                    first: Box::new(t1),
                    second: Box::new(t2),
                    span,
                }
            } else {
                t1
            }
        })
        .parse_next(input)
}

fn tensor(input: &mut LocatingSlice<&str>) -> Result<TermR<Range<usize>>> {
    (atom, opt(preceded((multispace0, 'x', multispace0), tensor)))
        .with_span()
        .map(|((t1, rest), span)| {
            if let Some(t2) = rest {
                TermR::Tensor {
                    first: Box::new(t1),
                    second: Box::new(t2),
                    span,
                }
            } else {
                t1
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
	seq!(_: "if", _: multispace1, _: "let", _: multispace1, pattern, _: multispace1, _: "then", _: multispace1, tensor).with_span().map(|((pattern, inner), span)| TermR::IfLet{ pattern, inner: Box::new(inner), span })
    )).parse_next(input)
}

fn pattern(input: &mut LocatingSlice<&str>) -> Result<PatternR<Range<usize>>> {
    (
        pattern_tensor,
        opt(preceded((multispace0, '.', multispace0), pattern)),
    )
        .with_span()
        .map(|((t1, rest), span)| {
            if let Some(t2) = rest {
                PatternR::Comp {
                    first: Box::new(t1),
                    second: Box::new(t2),
                    span,
                }
            } else {
                t1
            }
        })
        .parse_next(input)
}

fn pattern_tensor(input: &mut LocatingSlice<&str>) -> Result<PatternR<Range<usize>>> {
    (
        pattern_atom,
        opt(preceded((multispace0, 'x', multispace0), pattern_tensor)),
    )
        .with_span()
        .map(|((t1, rest), span)| {
            if let Some(t2) = rest {
                PatternR::Tensor {
                    first: Box::new(t1),
                    second: Box::new(t2),
                    span,
                }
            } else {
                t1
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
