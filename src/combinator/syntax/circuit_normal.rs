//! Definitions of circuit-normal syntax.
//!
//! Circuit-normal syntax is designed to allow extraction to Hadamard/Controlled phase circuits.

use super::{
    KetState, Phase,
    typed::{PatternT, TermT, TermType},
};

/// Circuit-normal terms.
///
/// These takes the form:
/// if let q_11 x ... x q_1n then Phase(theta_1) x id(m_1);
/// ... ;
/// if let q_l1 x ... x q_ln then Phase(theta_l) x id(m_l)
#[derive(Clone, Debug, PartialEq)]
pub struct TermC {
    pub(crate) clauses: Vec<ClauseC>,
    pub(crate) ty: TermType,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ClauseC {
    pub(crate) pattern: PatternC,
    pub(crate) phase: f64,
}


#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PatternC {
    pub(crate) parts: Vec<Option<KetState>>,
}

impl TermC {
    /// Return a `TermT` which is the "quotation" of this circuit-normal-form term.
    /// Realises that all circuit-normal-form terms are also terms.
    pub fn quote(&self) -> TermT {
        match self.clauses.len() {
            0 => TermT::Id {
                ty: self.ty,
            },
            1 => self.clauses[0].quote(),
            _ => TermT::Comp {
                terms: self.clauses.iter().map(ClauseC::quote).collect(),
                ty: self.ty,
            },
        }
    }
}

impl ClauseC {
    pub(crate) fn quote(&self) -> TermT {
	let id_qubits = self.pattern.id_qubits();
        let mut inner = TermT::Phase {
            phase: Phase::Angle(self.phase),
        };
        if id_qubits != 0 {
            inner = TermT::Tensor {
                terms: vec![
                    inner,
                    TermT::Id {
                        ty: TermType(id_qubits),
                    },
                ],
            }
        }

        TermT::IfLet {
            pattern: self.pattern.quote(),
            inner: Box::new(inner),
        }
    }

    pub(crate) fn invert(&self) -> ClauseC {
        ClauseC {
            pattern: self.pattern.clone(),
            phase: -self.phase,
        }
    }
}

fn state_to_pattern(s: Option<KetState>) -> PatternT {
    s.map_or(
        PatternT::Unitary(Box::new(TermT::Id { ty: TermType(1) })),
        |state| PatternT::Ket {
            states: vec![state],
        },
    )
}

impl PatternC {
    pub(crate) fn quote(&self) -> PatternT {
        if self.parts.len() == 1 {
            state_to_pattern(self.parts[0])
        } else {
            PatternT::Tensor {
                patterns: self.parts.iter().cloned().map(state_to_pattern).collect(),
            }
        }
    }

    pub(crate) fn id_qubits(&self) -> usize {
	self.parts.iter().filter(|x| x.is_none()).count()
    }

    pub(crate) fn id(l: usize) -> Self {
        PatternC {
            parts: vec![None; l],
        }
    }
}
